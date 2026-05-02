use std::collections::{BTreeSet, HashMap};
use std::fs;
use std::path::{Component, Path, PathBuf};

#[derive(Debug, Clone)]
pub struct LuaFile {
    pub source_path: PathBuf,
    pub archive_path: String,
}

#[derive(Debug)]
struct GameEntry {
    terms: Vec<String>,
    files: Vec<LuaFile>,
}

/// In-memory index of all .lua files organized by game folder.
pub struct LuaFinder {
    games: Vec<GameEntry>,
    by_app_id: HashMap<String, usize>,
    file_count: usize,
}

impl LuaFinder {
    /// Scan `base_dir` and build an index of all .lua files.
    ///
    /// Expected structure: `base_dir/<AppID>/<file>.lua`. Nested folders under
    /// each AppID are supported and preserved inside the ZIP archive.
    pub fn new(base_dir: &str) -> Self {
        let mut games = Vec::new();
        let mut by_app_id = HashMap::new();
        let mut file_count = 0;

        let base = Path::new(base_dir);
        if !base.exists() {
            log::warn!("lua_files directory not found at: {}", base.display());
            return Self {
                games,
                by_app_id,
                file_count,
            };
        }

        let entries = match fs::read_dir(base) {
            Ok(entries) => entries,
            Err(err) => {
                log::error!("Failed to read {}: {}", base.display(), err);
                return Self {
                    games,
                    by_app_id,
                    file_count,
                };
            }
        };

        for entry in entries.flatten() {
            let Ok(file_type) = entry.file_type() else {
                continue;
            };
            if !file_type.is_dir() {
                continue;
            }

            let app_dir = entry.path();
            let Some(app_id) = app_dir.file_name().and_then(|name| name.to_str()) else {
                continue;
            };

            let mut files = Vec::new();
            collect_lua_files(app_id, &app_dir, &app_dir, &mut files);

            if files.is_empty() {
                continue;
            }

            let index = games.len();
            let terms = build_terms(app_id, &files);
            file_count += files.len();
            by_app_id.insert(app_id.to_owned(), index);

            log::info!("Indexed {app_id}: {} .lua file(s)", files.len());
            games.push(GameEntry { terms, files });
        }

        Self {
            games,
            by_app_id,
            file_count,
        }
    }

    /// Search for .lua files by AppID, known game alias, folder name, or file name.
    pub fn search(&self, query: &str) -> Vec<LuaFile> {
        let trimmed = query.trim();
        if trimmed.is_empty() {
            return Vec::new();
        }

        if let Some(index) = self.by_app_id.get(trimmed) {
            return self.games[*index].files.clone();
        }

        let normalized_query = normalize(trimmed);
        if normalized_query.is_empty() {
            return Vec::new();
        }

        let mut matched_indexes = BTreeSet::new();
        for (index, game) in self.games.iter().enumerate() {
            if game
                .terms
                .iter()
                .any(|term| term_matches(term, &normalized_query))
            {
                matched_indexes.insert(index);
            }
        }

        let mut results = Vec::new();
        for index in matched_indexes {
            results.extend(self.games[index].files.iter().cloned());
        }

        results
    }

    pub fn app_count(&self) -> usize {
        self.games.len()
    }

    pub fn file_count(&self) -> usize {
        self.file_count
    }
}

fn collect_lua_files(app_id: &str, root: &Path, dir: &Path, out: &mut Vec<LuaFile>) {
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(err) => {
            log::warn!("Skipping unreadable directory {}: {}", dir.display(), err);
            return;
        }
    };

    for entry in entries.flatten() {
        let Ok(file_type) = entry.file_type() else {
            continue;
        };
        let path = entry.path();

        if file_type.is_dir() {
            collect_lua_files(app_id, root, &path, out);
            continue;
        }

        if !file_type.is_file() || !is_lua_file(&path) {
            continue;
        }

        let archive_path = match path.strip_prefix(root) {
            Ok(relative) => build_archive_path(app_id, relative),
            Err(_) => build_archive_path(app_id, Path::new("script.lua")),
        };

        out.push(LuaFile {
            source_path: path,
            archive_path,
        });
    }
}

fn build_terms(app_id: &str, files: &[LuaFile]) -> Vec<String> {
    let mut terms = BTreeSet::new();
    insert_term(&mut terms, app_id);

    for alias in known_aliases(app_id) {
        insert_term(&mut terms, alias);
    }

    for file in files {
        if let Some(stem) = file.source_path.file_stem().and_then(|stem| stem.to_str()) {
            insert_term(&mut terms, stem);
        }
    }

    terms.into_iter().collect()
}

fn known_aliases(app_id: &str) -> &'static [&'static str] {
    match app_id {
        "440" => &["Team Fortress 2", "TF2"],
        "570" => &["Dota 2", "DOTA"],
        "730" => &[
            "Counter-Strike 2",
            "Counter Strike 2",
            "CS2",
            "Counter-Strike Global Offensive",
            "CSGO",
        ],
        _ => &[],
    }
}

fn insert_term(terms: &mut BTreeSet<String>, value: &str) {
    let value = normalize(value);
    if !value.is_empty() {
        terms.insert(value);
    }
}

fn term_matches(term: &str, query: &str) -> bool {
    term == query || term.contains(query) || (query.len() >= 3 && query.contains(term))
}

fn is_lua_file(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("lua"))
}

fn build_archive_path(app_id: &str, relative_path: &Path) -> String {
    let mut parts = vec![sanitize_zip_component(app_id, "game")];

    for component in relative_path.components() {
        if let Component::Normal(value) = component {
            if let Some(value) = value.to_str() {
                parts.push(sanitize_zip_component(value, "script"));
            }
        }
    }

    if parts.len() == 1 {
        parts.push("script.lua".to_owned());
    }

    parts.join("/")
}

fn sanitize_zip_component(value: &str, fallback: &str) -> String {
    let sanitized: String = value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '.' | '-' | '_') {
                ch
            } else {
                '_'
            }
        })
        .collect();

    let sanitized = sanitized.trim_matches('_');
    if sanitized.is_empty() {
        fallback.to_owned()
    } else {
        sanitized.to_owned()
    }
}

fn normalize(value: &str) -> String {
    value
        .chars()
        .filter_map(|ch| {
            if ch.is_ascii_alphanumeric() {
                Some(ch.to_ascii_lowercase())
            } else if ch.is_whitespace() || matches!(ch, '-' | '_' | ':' | '.') {
                Some(' ')
            } else {
                None
            }
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}
