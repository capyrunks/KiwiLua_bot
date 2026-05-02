use teloxide::dispatching::{HandlerExt, UpdateFilterExt, UpdateHandler};
use teloxide::dptree;
use teloxide::types::Update;

use super::callbacks::handle_callback;
use super::commands::{cmd_help, cmd_language, cmd_start, handle_text, Command};

/// Build the dptree dispatcher schema
pub fn schema() -> UpdateHandler<Box<dyn std::error::Error + Send + Sync + 'static>> {
    let command_handler = dptree::entry()
        .filter_command::<Command>()
        .branch(dptree::case![Command::Start].endpoint(cmd_start))
        .branch(dptree::case![Command::Language].endpoint(cmd_language))
        .branch(dptree::case![Command::Help].endpoint(cmd_help));

    let message_handler = Update::filter_message()
        .branch(command_handler)
        .branch(dptree::endpoint(handle_text));

    let callback_handler = Update::filter_callback_query().endpoint(handle_callback);

    dptree::entry()
        .branch(message_handler)
        .branch(callback_handler)
}
