// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use std::time::Duration;

use anki_proto::ankiweb::CheckForUpdateRequest;
use anki_proto::ankiweb::CheckForUpdateResponse;
use anki_proto::ankiweb::GetAddonInfoRequest;
use anki_proto::ankiweb::GetAddonInfoResponse;
use prost::Message;
use reqwest::Client;

use crate::prelude::*;

fn service_url(service: &str) -> String {
    format!("https://ankiweb.net/svc/{service}")
}

async fn post<I, O>(client: &Client, service: &str, input: I) -> Result<O>
where
    I: Message,
    O: Message + Default,
{
    let out = client
        .post(service_url(service))
        .body(input.encode_to_vec())
        .timeout(Duration::from_secs(60))
        .send()
        .await?
        .error_for_status()?
        .bytes()
        .await?;
    let out: O = O::decode(&out[..])?;
    Ok(out)
}

pub async fn get_addon_info(
    client: &Client,
    input: GetAddonInfoRequest,
) -> Result<GetAddonInfoResponse> {
    post(client, "desktop/addon-info", input).await
}

pub async fn check_for_update(
    client: &Client,
    input: CheckForUpdateRequest,
) -> Result<CheckForUpdateResponse> {
    post(client, "desktop/check-for-update", input).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn addon_info() -> Result<()> {
        if std::env::var("ONLINE_TESTS").is_err() {
            println!("test disabled; ONLINE_TESTS not set");
            return Ok(());
        }
        let client = Client::new();
        let info = get_addon_info(
            &client,
            GetAddonInfoRequest {
                client_version: 30,
                addon_ids: vec![3918629684],
            },
        )
        .await?;
        assert_eq!(info.info[0].min_version, 0);
        assert_eq!(info.info[0].max_version, 49);
        Ok(())
    }
}
