use std::sync::Arc;

use teloxide::prelude::*;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup, InputFile};
use teloxide::utils::command::BotCommands;

use crate::i18n::{texts, Lang, LangStore};
use crate::search::finder::LuaFinder;
use crate::zip::packer;

type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

const MAX_QUERY_CHARS: usize = 128;

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
    lua_finder: Arc<LuaFinder>,
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

    if text.chars().count() > MAX_QUERY_CHARS {
        bot.send_message(msg.chat.id, texts::query_too_long(lang))
            .await?;
        return Ok(());
    }

    let files = lua_finder.search(text);
    if files.is_empty() {
        bot.send_message(msg.chat.id, texts::not_found(lang))
            .await?;
        return Ok(());
    }

    bot.send_message(msg.chat.id, texts::sending_files(lang))
        .await?;

    let archive_name = format!("{}.zip", archive_stem(text));
    let pack_result = tokio::task::spawn_blocking(move || packer::pack_files(&files)).await;

    match pack_result {
        Ok(Ok(zip_bytes)) => {
            let input_file = InputFile::memory(zip_bytes).file_name(archive_name);
            bot.send_document(msg.chat.id, input_file).await?;
        }
        Ok(Err(err)) => {
            log::error!("Failed to pack files for query {:?}: {}", text, err);
            bot.send_message(msg.chat.id, texts::packing_error(lang))
                .await?;
        }
        Err(err) => {
            log::error!("Packing task failed for query {:?}: {}", text, err);
            bot.send_message(msg.chat.id, texts::packing_error(lang))
                .await?;
        }
    }

    Ok(())
}

fn archive_stem(query: &str) -> String {
    let sanitized: String = query
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_') {
                ch
            } else {
                '_'
            }
        })
        .take(64)
        .collect();

    let sanitized = sanitized.trim_matches('_');
    if sanitized.is_empty() {
        "kiwilua".to_owned()
    } else {
        sanitized.to_owned()
    }
}
