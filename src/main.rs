mod bot;
mod i18n;
mod search;
mod zip;

use std::env;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use axum::routing::get;
use axum::Router;
use teloxide::prelude::*;
use tokio::task::JoinError;

use crate::bot::schema::schema;
use crate::i18n::LangStore;
use crate::search::finder::LuaFinder;

type AppError = Box<dyn std::error::Error + Send + Sync + 'static>;
type AppResult<T> = Result<T, AppError>;

const DEFAULT_HTTP_PORT: u16 = 7860;
const LUA_FILES_DIR: &str = "./lua_files";

/// Health check endpoint for Hugging Face Spaces.
/// Hugging Face pings this (or /) to know the container is alive.
async fn health() -> &'static str {
    "OK"
}

#[tokio::main]
async fn main() -> AppResult<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    log::info!("Starting KiwiLua Bot");

    let http_task = tokio::spawn(run_http_server());
    let bot_task = tokio::spawn(run_telegram_bot());

    tokio::select! {
        result = http_task => finish_task("HTTP health server", result)?,
        result = bot_task => finish_task("Telegram bot", result)?,
    }

    Ok(())
}

async fn run_http_server() -> AppResult<()> {
    let port = env::var("PORT")
        .ok()
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(DEFAULT_HTTP_PORT);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let app = Router::new()
        .route("/", get(health))
        .route("/health", get(health));

    let listener = tokio::net::TcpListener::bind(addr).await?;
    log::info!("HTTP health server listening on http://{addr}");

    axum::serve(listener, app).await?;
    Ok(())
}

async fn run_telegram_bot() -> AppResult<()> {
    let lang_store = Arc::new(LangStore::new(default_lang_store_path()));
    let lua_finder = Arc::new(LuaFinder::new(LUA_FILES_DIR));

    log::info!(
        "Indexed {} game folder(s) and {} .lua file(s)",
        lua_finder.app_count(),
        lua_finder.file_count()
    );

    let bot = Bot::new(telegram_token()?);

    Dispatcher::builder(bot, schema())
        .dependencies(dptree::deps![lang_store, lua_finder])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;

    Ok(())
}

fn telegram_token() -> AppResult<String> {
    let token = env::var("TELOXIDE_TOKEN").or_else(|_| env::var("BOT_TOKEN"))?;
    let token = token.trim().to_owned();

    if token.is_empty() {
        return Err(boxed_error(
            "Telegram token is empty. Set TELOXIDE_TOKEN or BOT_TOKEN.",
        ));
    }

    Ok(token)
}

fn default_lang_store_path() -> Option<PathBuf> {
    match env::var("LANG_STORE_PATH") {
        Ok(value) if value.eq_ignore_ascii_case("off") || value.eq_ignore_ascii_case("none") => {
            None
        }
        Ok(value) if !value.trim().is_empty() => Some(PathBuf::from(value)),
        _ if Path::new("/data").is_dir() => Some(PathBuf::from("/data/kiwilua_languages.db")),
        _ => Some(PathBuf::from("./data/kiwilua_languages.db")),
    }
}

fn finish_task(task: &str, result: Result<AppResult<()>, JoinError>) -> AppResult<()> {
    match result {
        Ok(Ok(())) => {
            log::warn!("{task} stopped");
            Ok(())
        }
        Ok(Err(err)) => {
            log::error!("{task} failed: {err}");
            Err(err)
        }
        Err(err) => Err(boxed_error(format!(
            "{task} task panicked or was cancelled: {err}"
        ))),
    }
}

fn boxed_error(message: impl Into<String>) -> AppError {
    Box::new(std::io::Error::other(message.into()))
}
