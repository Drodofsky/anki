// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use std::fmt::Display;
use std::io::ErrorKind;

use bytes::Bytes;
use futures::Stream;
use futures::TryStreamExt;
use http::HeaderName;
use http::StatusCode;
use serde::Deserialize;
use serde::Serialize;
use tokio_util::io::ReaderStream;

use crate::sync::error::HttpError;
use crate::sync::error::HttpResult;
use crate::sync::version::SyncVersion;

/// Does not enforce payload size
pub fn decode_zstd_body_stream_for_client<S, E>(data: S) -> impl Stream<Item = HttpResult<Bytes>>
where
    S: Stream<Item = Result<Bytes, E>> + Unpin,
    E: Display,
{
    let reader = tokio_util::io::StreamReader::new(
        data.map_err(|e| std::io::Error::new(ErrorKind::ConnectionAborted, format!("{e}"))),
    );
    let reader = async_compression::tokio::bufread::ZstdDecoder::new(reader);
    ReaderStream::new(reader).map_err(|err| HttpError {
        code: StatusCode::BAD_REQUEST,
        context: "decode zstd body".into(),
        source: Some(Box::new(err) as _),
    })
}

pub fn encode_zstd_body_stream<S, E>(data: S) -> impl Stream<Item = HttpResult<Bytes>>
where
    S: Stream<Item = Result<Bytes, E>> + Unpin,
    E: Display,
{
    let reader = tokio_util::io::StreamReader::new(
        data.map_err(|e| std::io::Error::new(ErrorKind::ConnectionAborted, format!("{e}"))),
    );
    let reader = async_compression::tokio::bufread::ZstdEncoder::new(reader);
    ReaderStream::new(reader).map_err(|err| HttpError {
        code: StatusCode::BAD_REQUEST,
        context: "encode zstd body".into(),
        source: Some(Box::new(err) as _),
    })
}

#[derive(Serialize, Deserialize)]
pub struct SyncHeader {
    #[serde(rename = "v")]
    pub sync_version: SyncVersion,
    #[serde(rename = "k")]
    pub sync_key: String,
    #[serde(rename = "c")]
    pub client_ver: String,
    #[serde(rename = "s")]
    pub session_key: String,
}

pub static SYNC_HEADER_NAME: HeaderName = HeaderName::from_static("anki-sync");
