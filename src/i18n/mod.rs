pub mod texts;

use dashmap::DashMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use teloxide::types::ChatId;

/// Supported languages.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Lang {
    En,
    Es,
    Tr,
    It,
    Fr,
    De,
    Ru,
}

impl Lang {
    pub const ALL: [Self; 7] = [
        Self::En,
        Self::Es,
        Self::Tr,
        Self::It,
        Self::Fr,
        Self::De,
        Self::Ru,
    ];

    pub fn from_code(code: &str) -> Option<Self> {
        match code {
            "en" => Some(Self::En),
            "es" => Some(Self::Es),
            "tr" => Some(Self::Tr),
            "it" => Some(Self::It),
            "fr" => Some(Self::Fr),
            "de" => Some(Self::De),
            "ru" => Some(Self::Ru),
            _ => None,
        }
    }

    pub fn code(self) -> &'static str {
        match self {
            Self::En => "en",
            Self::Es => "es",
            Self::Tr => "tr",
            Self::It => "it",
            Self::Fr => "fr",
            Self::De => "de",
            Self::Ru => "ru",
        }
    }

    pub fn display_name(self) -> &'static str {
        match self {
            Self::En => "🇬🇧 English",
            Self::Es => "🇪🇸 Español",
            Self::Tr => "🇹🇷 Türkçe",
            Self::It => "🇮🇹 Italiano",
            Self::Fr => "🇫🇷 Français",
            Self::De => "🇩🇪 Deutsch",
            Self::Ru => "🇷🇺 Русский",
        }
    }
}

/// Thread-safe storage for user language preferences.
///
/// The in-memory map keeps handlers fast. If a path is configured, every
/// language change is also flushed to a tiny line-based file:
/// `<chat_id> <lang_code>`.
pub struct LangStore {
    map: DashMap<ChatId, Lang>,
    path: Option<PathBuf>,
}

impl LangStore {
    pub fn new(path: Option<PathBuf>) -> Self {
        let store = Self {
            map: DashMap::new(),
            path,
        };

        if let Some(path) = store.path.as_deref() {
            match store.load(path) {
                Ok(count) => log::info!("Loaded {count} saved language preference(s)"),
                Err(err) if err.kind() == io::ErrorKind::NotFound => {
                    log::info!("Language state file will be created at {}", path.display());
                }
                Err(err) => log::warn!(
                    "Could not load language state from {}: {}",
                    path.display(),
                    err
                ),
            }
        } else {
            log::info!("Language persistence disabled by LANG_STORE_PATH");
        }

        store
    }

    pub fn get(&self, chat_id: ChatId) -> Option<Lang> {
        self.map.get(&chat_id).map(|value| *value)
    }

    pub fn set(&self, chat_id: ChatId, lang: Lang) -> io::Result<()> {
        self.map.insert(chat_id, lang);
        self.flush()
    }

    fn load(&self, path: &Path) -> io::Result<usize> {
        let content = fs::read_to_string(path)?;
        let mut loaded = 0;

        for line in content.lines() {
            let mut parts = line.split_whitespace();
            let Some(chat_id) = parts.next() else {
                continue;
            };
            let Some(lang_code) = parts.next() else {
                continue;
            };

            let Ok(chat_id) = chat_id.parse::<i64>() else {
                continue;
            };
            let Some(lang) = Lang::from_code(lang_code) else {
                continue;
            };

            self.map.insert(ChatId(chat_id), lang);
            loaded += 1;
        }

        Ok(loaded)
    }

    fn flush(&self) -> io::Result<()> {
        let Some(path) = self.path.as_deref() else {
            return Ok(());
        };

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let mut entries: Vec<(i64, &'static str)> = self
            .map
            .iter()
            .map(|entry| (entry.key().0, entry.value().code()))
            .collect();
        entries.sort_unstable_by_key(|(chat_id, _)| *chat_id);

        let mut body = String::new();
        for (chat_id, code) in entries {
            body.push_str(&format!("{chat_id} {code}\n"));
        }

        let tmp_path = path.with_extension("tmp");
        fs::write(&tmp_path, body)?;

        if let Err(err) = fs::rename(&tmp_path, path) {
            if path.exists() {
                fs::remove_file(path)?;
                fs::rename(&tmp_path, path)
            } else {
                Err(err)
            }
        } else {
            Ok(())
        }
    }
}
