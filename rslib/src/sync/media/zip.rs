// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use std::io;
use std::io::Write;

use serde::Deserialize;
use serde_tuple::Serialize_tuple;
use zip::write::FileOptions;
use zip::ZipWriter;

use crate::prelude::*;

pub struct ZipFileMetadata {
    pub filename: String,
    pub total_bytes: u32,
    pub sha1: String,
}

pub fn zip_files_for_upload(entries_: Vec<(String, Option<Vec<u8>>)>) -> Result<Vec<u8>> {
    let options: FileOptions<'_, ()> =
        FileOptions::default().compression_method(zip::CompressionMethod::Stored);
    let mut zip = ZipWriter::new(io::Cursor::new(vec![]));
    let mut entries = vec![];

    for (idx, (filename, data)) in entries_.into_iter().enumerate() {
        match data {
            None => {
                entries.push(UploadEntry {
                    actual_filename: filename,
                    filename_in_zip: None,
                });
            }
            Some(data) => {
                let idx_str = idx.to_string();
                zip.start_file(&idx_str, options)?;
                zip.write_all(&data)?;
                entries.push(UploadEntry {
                    actual_filename: filename,
                    filename_in_zip: Some(idx_str),
                });
            }
        }
    }

    let meta = serde_json::to_vec(&entries)?;
    zip.start_file("_meta", options)?;
    zip.write_all(&meta)?;

    Ok(zip.finish()?.into_inner())
}

#[derive(Serialize_tuple, Deserialize)]
struct UploadEntry {
    actual_filename: String,
    filename_in_zip: Option<String>,
}
