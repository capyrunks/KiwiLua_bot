use std::sync::Arc;

use teloxide::prelude::*;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup, InputFile};
use teloxide::utils::command::BotCommands;

use crate::i18n::{texts, Lang, LangStore};
use crate::source::{self, FetchError, FetchedKind, LuaSourceConfig};
use crate::zip::packer;

type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

const MAX_APP_ID_DIGITS: usize = 10;

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase")]
pub enum Command {
    #[command(description = "Start the bot")]
    Start,
    #[command(description = "Change language")]
    Language,
    #[command(description = "Get help")]
    Help,
}

pub fn language_keyboard() -> InlineKeyboardMarkup {
    let buttons: Vec<Vec<InlineKeyboardButton>> = Lang::ALL
        .chunks(2)
        .map(|chunk| {
            chunk
                .iter()
                .map(|lang| {
                    InlineKeyboardButton::callback(
                        lang.display_name().to_owned(),
                        format!("lang:{}", lang.code()),
                    )
                })
                .collect()
        })
        .collect();

    InlineKeyboardMarkup::new(buttons)
}

pub async fn cmd_start(bot: Bot, msg: Message, lang_store: Arc<LangStore>) -> HandlerResult {
    if let Some(lang) = lang_store.get(msg.chat.id) {
        bot.send_message(msg.chat.id, texts::ready(lang)).await?;
    } else {
        bot.send_message(msg.chat.id, texts::choose_language_initial())
            .reply_markup(language_keyboard())
            .await?;
    }

    Ok(())
}

pub async fn cmd_language(bot: Bot, msg: Message, lang_store: Arc<LangStore>) -> HandlerResult {
    let lang = lang_store.get(msg.chat.id).unwrap_or(Lang::En);

    bot.send_message(msg.chat.id, texts::choose_language(lang))
        .reply_markup(language_keyboard())
        .await?;

    Ok(())
}

pub async fn cmd_help(bot: Bot, msg: Message, lang_store: Arc<LangStore>) -> HandlerResult {
    let lang = lang_store.get(msg.chat.id).unwrap_or(Lang::En);

    bot.send_message(msg.chat.id, texts::help_text(lang))
        .await?;

    Ok(())
}

pub async fn handle_text(
    bot: Bot,
    msg: Message,
    lang_store: Arc<LangStore>,
    source_config: Arc<LuaSourceConfig>,
    http_client: reqwest::Client,
) -> HandlerResult {
    let text = match msg.text().map(str::trim) {
        Some(text) if !text.is_empty() => text,
        _ => return Ok(()),
    };

    let lang = match lang_store.get(msg.chat.id) {
        Some(lang) => lang,
        None => {
            bot.send_message(msg.chat.id, texts::no_language_set())
                .reply_markup(language_keyboard())
                .await?;
            return Ok(());
        }
    };

    let Some(app_id) = parse_app_id(text) else {
        bot.send_message(msg.chat.id, texts::app_id_only(lang))
            .await?;
        return Ok(());
    };

    bot.send_message(msg.chat.id, texts::fetching_config(lang))
        .await?;

    let fetched = match source::fetch_config(&http_client, source_config.as_ref(), &app_id).await {
        Ok(config) => config,
        Err(FetchError::NotFound { attempts }) => {
            log::info!("Config for AppID {} not found: {:?}", app_id, attempts);
            bot.send_message(msg.chat.id, texts::not_found(lang))
                .await?;
            return Ok(());
        }
        Err(FetchError::Unavailable { attempts }) => {
            log::error!(
                "All Lua sources failed for AppID {}: {:?}",
                app_id,
                attempts
            );
            bot.send_message(msg.chat.id, texts::source_unavailable(lang))
                .await?;
            return Ok(());
        }
    };

    log::info!(
        "Fetched AppID {} config from {} as {:?}",
        app_id,
        fetched.source_url,
        fetched.kind
    );

    let archive_name = format!("{app_id}.zip");
    let archive_bytes = match fetched.kind {
        FetchedKind::Zip => fetched.bytes,
        FetchedKind::Lua => {
            let lua_bytes = fetched.bytes;
            let app_id_for_zip = app_id.clone();
            match tokio::task::spawn_blocking(move || {
                packer::pack_lua_from_memory(&app_id_for_zip, &lua_bytes)
            })
            .await
            {
                Ok(Ok(zip_bytes)) => zip_bytes,
                Ok(Err(err)) => {
                    log::error!("Failed to pack Lua for AppID {}: {}", app_id, err);
                    bot.send_message(msg.chat.id, texts::packing_error(lang))
                        .await?;
                    return Ok(());
                }
                Err(err) => {
                    log::error!("Packing task failed for AppID {}: {}", app_id, err);
                    bot.send_message(msg.chat.id, texts::packing_error(lang))
                        .await?;
                    return Ok(());
                }
            }
        }
    };

    let input_file = InputFile::memory(archive_bytes).file_name(archive_name);
    bot.send_document(msg.chat.id, input_file).await?;

    Ok(())
}

fn parse_app_id(text: &str) -> Option<String> {
    let trimmed = text.trim();

    if trimmed.is_empty()
        || trimmed.len() > MAX_APP_ID_DIGITS
        || !trimmed.chars().all(|ch| ch.is_ascii_digit())
    {
        return None;
    }

    Some(trimmed.to_owned())
}
