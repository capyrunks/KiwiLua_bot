use std::collections::HashSet;
use std::fs::{self, File};
use std::io::{self, BufReader, Cursor};

use zip::write::SimpleFileOptions;
use zip::CompressionMethod;
use zip::ZipWriter;

use crate::search::finder::LuaFile;

type PackResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync + 'static>>;

const MAX_FILES_PER_ARCHIVE: usize = 256;
const MAX_UNCOMPRESSED_BYTES: u64 = 50 * 1024 * 1024;

/// Pack files into a ZIP archive entirely in memory.
///
/// Source files are streamed from disk into `ZipWriter<Cursor<Vec<u8>>>`; no
/// temporary archive file is created on disk.
pub fn pack_files(files: &[LuaFile]) -> PackResult<Vec<u8>> {
    validate_input(files)?;

    let cursor = Cursor::new(Vec::new());
    let mut zip = ZipWriter::new(cursor);
    let mut used_names = HashSet::new();
    let options = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Deflated)
        .unix_permissions(0o644);

    for file in files {
        let archive_path = unique_archive_path(&file.archive_path, &mut used_names);
        let source = File::open(&file.source_path)?;
        let mut source = BufReader::new(source);

        zip.start_file(&archive_path, options)?;
        io::copy(&mut source, &mut zip)?;

        log::debug!("Added {} as {}", file.source_path.display(), archive_path);
    }

    let cursor = zip.finish()?;
    let bytes = cursor.into_inner();

    log::info!(
        "ZIP created in memory: {} file(s), {} byte(s)",
        files.len(),
        bytes.len()
    );

    Ok(bytes)
}

fn validate_input(files: &[LuaFile]) -> PackResult<()> {
    if files.is_empty() {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "no files to pack").into());
    }

    if files.len() > MAX_FILES_PER_ARCHIVE {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!(
                "too many files: {} > {}",
                files.len(),
                MAX_FILES_PER_ARCHIVE
            ),
        )
        .into());
    }

    let mut total_size = 0_u64;
    for file in files {
        let metadata = fs::metadata(&file.source_path)?;
        if !metadata.is_file() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("not a regular file: {}", file.source_path.display()),
            )
            .into());
        }

        total_size = total_size
            .checked_add(metadata.len())
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "archive size overflow"))?;

        if total_size > MAX_UNCOMPRESSED_BYTES {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!(
                    "archive too large: {} bytes > {} bytes",
                    total_size, MAX_UNCOMPRESSED_BYTES
                ),
            )
            .into());
        }
    }

    Ok(())
}

fn unique_archive_path(path: &str, used_names: &mut HashSet<String>) -> String {
    if used_names.insert(path.to_owned()) {
        return path.to_owned();
    }

    for index in 2.. {
        let candidate = append_suffix(path, index);
        if used_names.insert(candidate.clone()) {
            return candidate;
        }
    }

    unreachable!("unbounded suffix loop must eventually find a unique name")
}

fn append_suffix(path: &str, index: usize) -> String {
    match path.rfind('.') {
        Some(dot_index) => format!("{}_{}{}", &path[..dot_index], index, &path[dot_index..]),
        None => format!("{path}_{index}"),
    }
}
