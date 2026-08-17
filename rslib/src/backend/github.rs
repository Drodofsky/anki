// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

//! The desktop self-updater (checking/downloading new Anki releases from
//! GitHub) was removed, as it's specific to the native desktop app. These
//! stubs remain because the protobuf-generated dispatch code requires
//! `Backend` to implement every `Backend*Service` trait.

use anki_proto::github::GithubRelease;
use anki_proto::github::LatestReleaseRequest;

use super::Backend;
use crate::prelude::*;
use crate::services::BackendGithubService;

impl BackendGithubService for Backend {
    fn get_latest_release(&self, _input: LatestReleaseRequest) -> Result<GithubRelease> {
        invalid_input!("self-updater not available in this build");
    }

    fn download_release(&self, _release: GithubRelease) -> Result<anki_proto::generic::String> {
        invalid_input!("self-updater not available in this build");
    }
}
