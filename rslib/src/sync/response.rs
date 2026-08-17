// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use std::marker::PhantomData;

use http::HeaderName;
use serde::de::DeserializeOwned;

use crate::prelude::*;
use crate::sync::collection::upload::UploadResponse;

pub static ORIGINAL_SIZE: HeaderName = HeaderName::from_static("anki-original-size");

/// Stores the data returned from a sync request, and the type
/// it represents. Given a SyncResponse<Foo>, you can get a Foo
/// struct via .json(), except for uploads/downloads.
#[derive(Debug)]
pub struct SyncResponse<T> {
    pub data: Vec<u8>,
    json_output_type: PhantomData<T>,
}

impl<T> SyncResponse<T> {
    pub fn from_vec(data: Vec<u8>) -> SyncResponse<T> {
        SyncResponse {
            data,
            json_output_type: Default::default(),
        }
    }
}

impl SyncResponse<UploadResponse> {
    // Unfortunately the sync protocol sends this as a bare string
    // instead of JSON.
    pub fn upload_response(&self) -> UploadResponse {
        let resp = String::from_utf8_lossy(&self.data);
        match resp.as_ref() {
            "OK" => UploadResponse::Ok,
            other => UploadResponse::Err(other.into()),
        }
    }
}

impl<T> SyncResponse<T>
where
    T: DeserializeOwned,
{
    pub fn json(&self) -> Result<T> {
        serde_json::from_slice(&self.data).map_err(Into::into)
    }
}
