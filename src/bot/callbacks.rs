use std::sync::Arc;

use teloxide::prelude::*;

use crate::i18n::{texts, Lang, LangStore};

type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

pub async fn handle_callback(
    bot: Bot,
    q: CallbackQuery,
    lang_store: Arc<LangStore>,
) -> HandlerResult {
    let callback_id = q.id.clone();
    let Some(data) = q.data.as_deref() else {
        bot.answer_callback_query(callback_id).await?;
        return Ok(());
    };

    let Some(code) = data.strip_prefix("lang:") else {
        bot.answer_callback_query(callback_id).await?;
        return Ok(());
    };

    let Some(lang) = Lang::from_code(code) else {
        bot.answer_callback_query(callback_id)
            .text("Unsupported language")
            .show_alert(true)
            .await?;
        return Ok(());
    };

    let Some(message) = q.message.as_ref() else {
        bot.answer_callback_query(callback_id).await?;
        return Ok(());
    };

    let chat_id = message.chat().id;
    let message_id = message.id();

    if let Err(err) = lang_store.set(chat_id, lang) {
        log::warn!("Language was changed in memory but not persisted: {}", err);
    }

    bot.answer_callback_query(callback_id).await?;

    let confirmation = format!(
        "{}\n\n{}",
        texts::language_set(lang),
        texts::search_prompt(lang)
    );

    if let Err(err) = bot
        .edit_message_text(chat_id, message_id, confirmation.clone())
        .await
    {
        log::warn!("Could not edit language selection message: {}", err);
        bot.send_message(chat_id, confirmation).await?;
    }

    log::info!("User {} set language to {:?}", chat_id, lang);
    Ok(())
}
