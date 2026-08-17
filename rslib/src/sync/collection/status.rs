// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use anki_proto::sync::sync_status_response;
use anki_proto::sync::sync_status_response::Required;
use anki_proto::sync::SyncStatusResponse;
use reqwest::Client;
use tracing::debug;

use crate::error::SyncErrorKind;
use crate::prelude::*;
use crate::sync::collection::meta::SyncMeta;
use crate::sync::collection::normal::ClientSyncState;
use crate::sync::http_client::HttpSyncClient;
use crate::sync::login::SyncAuth;

impl Collection {
    /// Checks local collection only. If local collection is clean but changes
    /// are pending on AnkiWeb, NoChanges will be returned.
    pub fn sync_status_offline(&mut self) -> Result<sync_status_response::Required> {
        let stamps = self.storage.get_collection_timestamps()?;
        let required = if stamps.never_synced() {
            // A collection that has never synced can't know its true state without
            // consulting the server; reporting FullSync here would otherwise stick
            // permanently when both sides are empty (mod=0 on both ends).
            sync_status_response::Required::NoChanges
        } else if stamps.schema_changed_since_sync() {
            sync_status_response::Required::FullSync
        } else if stamps.collection_changed_since_sync() {
            sync_status_response::Required::NormalSync
        } else {
            sync_status_response::Required::NoChanges
        };

        Ok(required)
    }
}

/// Should be called if a call to sync_status_offline() returns NoChanges, to
/// check if AnkiWeb has pending changes. Caller should persist new endpoint if
/// returned.
///
/// This routine is outside of the collection, as we don't want to block
/// collection access for a potentially slow network request that happens in the
/// background.
pub async fn online_sync_status_check(
    local: SyncMeta,
    server: &mut HttpSyncClient,
) -> Result<ClientSyncState, AnkiError> {
    let (remote, new_endpoint) = server.meta_with_redirect().await?;
    debug!(?remote, "meta");
    debug!(?local, "meta");
    if !remote.should_continue {
        debug!(remote.server_message, "server says abort");
        return Err(AnkiError::sync_error(
            remote.server_message,
            SyncErrorKind::ServerMessage,
        ));
    }
    let delta = remote.current_time.0 - local.current_time.0;
    if delta.abs() > 300 {
        debug!(delta, "clock off");
        return Err(AnkiError::sync_error("", SyncErrorKind::ClockIncorrect));
    }
    Ok(local.compared_to_remote(remote, new_endpoint))
}

/// Caches the result of a remote sync status check, so repeated calls
/// within a short window don't hit the network. Caller-owned: create one
/// and keep passing it into [check_sync_status].
#[derive(Default, Debug)]
pub struct RemoteSyncStatus {
    pub last_check: TimestampSecs,
    pub last_response: Required,
}

impl RemoteSyncStatus {
    pub fn update(&mut self, required: Required) {
        self.last_check = TimestampSecs::now();
        self.last_response = required;
    }
}

/// Checks whether a sync is required, consulting the network only if the
/// local collection has no pending changes and the cache has gone stale
/// (older than 300 seconds).
pub async fn check_sync_status(
    col: &mut Collection,
    auth: SyncAuth,
    client: Client,
    cache: &mut RemoteSyncStatus,
) -> Result<SyncStatusResponse> {
    // any local changes mean we can skip the network round-trip
    let req = col.sync_status_offline()?;
    if req != Required::NoChanges {
        return Ok(status_response_from_required(req));
    }

    // return cached server response if only a short time has elapsed
    if cache.last_check.elapsed_secs() < 300 {
        return Ok(status_response_from_required(cache.last_response));
    }

    // fetch and cache result
    let time_at_check_begin = TimestampSecs::now();
    let local = col.sync_meta()?;
    let mut http_client = HttpSyncClient::new(auth, client);
    let state = online_sync_status_check(local, &mut http_client).await?;
    if cache.last_check < time_at_check_begin {
        cache.last_check = time_at_check_begin;
        cache.last_response = state.required.into();
    }

    Ok(state.into())
}

fn status_response_from_required(required: Required) -> SyncStatusResponse {
    SyncStatusResponse {
        required: required.into(),
        new_endpoint: None,
    }
}

impl From<ClientSyncState> for SyncStatusResponse {
    fn from(r: ClientSyncState) -> Self {
        SyncStatusResponse {
            required: Required::from(r.required).into(),
            new_endpoint: r.new_endpoint,
        }
    }
}

impl From<crate::sync::collection::normal::SyncActionRequired> for Required {
    fn from(r: crate::sync::collection::normal::SyncActionRequired) -> Self {
        use crate::sync::collection::normal::SyncActionRequired;
        match r {
            SyncActionRequired::NoChanges => Required::NoChanges,
            SyncActionRequired::FullSyncRequired { .. } => Required::FullSync,
            SyncActionRequired::NormalSyncRequired => Required::NormalSync,
        }
    }
}
