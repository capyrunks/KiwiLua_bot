use std::io::{self, Cursor, Write};

use zip::write::SimpleFileOptions;
use zip::CompressionMethod;
use zip::ZipWriter;

type PackResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync + 'static>>;

const MAX_LUA_BYTES: usize = 50 * 1024 * 1024;

/// Pack one Lua file into a ZIP archive entirely in memory.
pub fn pack_lua_from_memory(app_id: &str, lua_bytes: &[u8]) -> PackResult<Vec<u8>> {
    validate_lua(app_id, lua_bytes)?;

    let cursor = Cursor::new(Vec::new());
    let mut zip = ZipWriter::new(cursor);
    let options = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Deflated)
        .unix_permissions(0o644);

    let file_name = format!("{}.lua", sanitize_app_id(app_id));
    zip.start_file(file_name, options)?;
    zip.write_all(lua_bytes)?;

    let cursor = zip.finish()?;
    let bytes = cursor.into_inner();

    log::info!(
        "ZIP created in memory for AppID {}: {} byte(s)",
        app_id,
        bytes.len()
    );

    Ok(bytes)
}

fn validate_lua(app_id: &str, lua_bytes: &[u8]) -> PackResult<()> {
    if app_id.is_empty() || !app_id.chars().all(|ch| ch.is_ascii_digit()) {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "invalid AppID").into());
    }

    if lua_bytes.is_empty() {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "empty Lua file").into());
    }

    if lua_bytes.len() > MAX_LUA_BYTES {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!(
                "Lua file too large: {} bytes > {} bytes",
                lua_bytes.len(),
                MAX_LUA_BYTES
            ),
        )
        .into());
    }

    std::str::from_utf8(lua_bytes)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "Lua file is not UTF-8 text"))?;

    Ok(())
}

fn sanitize_app_id(app_id: &str) -> String {
    app_id
        .chars()
        .filter(|ch| ch.is_ascii_digit())
        .take(10)
        .collect()
}

#[cfg(test)]
mod tests {
    use std::io::Read;

    use zip::ZipArchive;

    use super::*;

    #[test]
    fn packs_lua_bytes_without_touching_disk() {
        let archive_bytes = pack_lua_from_memory("730", b"addappid(730)")
            .expect("in-memory Lua packing should succeed");
        let cursor = Cursor::new(archive_bytes);
        let mut archive = ZipArchive::new(cursor).expect("ZIP should be readable");
        let mut file = archive.by_name("730.lua").expect("Lua file should exist");
        let mut body = String::new();

        file.read_to_string(&mut body)
            .expect("Lua file should be UTF-8 text");

        assert_eq!(body, "addappid(730)");
    }
}
