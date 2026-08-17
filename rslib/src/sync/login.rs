// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use reqwest::Client;
use reqwest::Url;
use serde::Deserialize;
use serde::Serialize;

use crate::prelude::*;
use crate::sync::collection::protocol::SyncProtocol;
use crate::sync::http_client::HttpSyncClient;
use crate::sync::request::IntoSyncRequest;

#[derive(Clone, Default)]
pub struct SyncAuth {
    pub hkey: String,
    pub endpoint: Option<Url>,
    pub io_timeout_secs: Option<u32>,
}

impl TryFrom<anki_proto::sync::SyncAuth> for SyncAuth {
    type Error = AnkiError;

    fn try_from(value: anki_proto::sync::SyncAuth) -> std::result::Result<Self, Self::Error> {
        Ok(SyncAuth {
            hkey: value.hkey,
            endpoint: value
                .endpoint
                .map(|v| {
                    Url::try_from(v.as_str())
                        // Without the next line, incomplete URLs like computer.local without the http://
                        // are detected but URLs like computer.local:8000 are not.
                        // By calling join() now, these URLs are detected too and later code that
                        // uses and unwraps the result of join() doesn't panic
                        .and_then(|x| x.join("./"))
                        .or_invalid("Invalid sync server specified. Please check the preferences.")
                })
                .transpose()?,
            io_timeout_secs: value.io_timeout_secs,
        })
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct HostKeyRequest {
    #[serde(rename = "u")]
    pub username: String,
    #[serde(rename = "p")]
    pub password: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct HostKeyResponse {
    pub key: String,
}

pub async fn sync_login<S: Into<String>>(
    username: S,
    password: S,
    endpoint: Option<String>,
    client: Client,
) -> Result<SyncAuth> {
    let auth = anki_proto::sync::SyncAuth {
        endpoint,
        ..Default::default()
    }
    .try_into()?;
    let client = HttpSyncClient::new(auth, client);
    let resp = client
        .host_key(
            HostKeyRequest {
                username: username.into(),
                password: password.into(),
            }
            .try_into_sync_request()?,
        )
        .await?
        .json()?;
    Ok(SyncAuth {
        hkey: resp.key,
        endpoint: None,
        io_timeout_secs: None,
    })
}
