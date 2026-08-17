// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use anki_io::atomic_rename;
use anki_io::new_tempfile_in_parent_of;
use anki_io::write_file;
use reqwest::Client;

use crate::collection::CollectionBuilder;
use crate::prelude::*;
use crate::sync::collection::protocol::EmptyInput;
use crate::sync::http_client::HttpSyncClient;
use crate::sync::login::SyncAuth;

impl Collection {
    /// Download collection from AnkiWeb. Caller must re-open afterwards.
    pub async fn full_download(self, auth: SyncAuth, client: Client) -> Result<()> {
        self.full_download_with_server(HttpSyncClient::new(auth, client))
            .await
    }

    // pub for tests
    pub(super) async fn full_download_with_server(self, server: HttpSyncClient) -> Result<()> {
        let col_path = self.col_path.clone();
        let _col_folder = col_path.parent().or_invalid("couldn't get col_folder")?;
        let progress = self.new_progress_handler();
        self.close(None)?;
        let out_data = server
            .download_with_progress(EmptyInput::request(), progress)
            .await?
            .data;
        // check file ok
        let temp_file = new_tempfile_in_parent_of(&col_path)?;
        write_file(temp_file.path(), out_data)?;
        let col = CollectionBuilder::new(temp_file.path())
            .set_check_integrity(true)
            .build()?;
        col.storage.db.execute_batch("update col set ls=mod")?;
        col.close(None)?;
        atomic_rename(temp_file, &col_path, true)?;
        Ok(())
    }
}
