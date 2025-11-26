use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Serialize, Debug)]
struct SearchMetadataRequest<'a> {
    checksum: &'a str,
}

#[derive(Deserialize, Debug)]
struct SearchMetadataResponse {
    assets: SearchMetadataResponseAssets,
}

#[derive(Deserialize, Debug)]
struct SearchMetadataResponseAssets {
    items: Vec<Asset>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Asset {
    pub id: String,

    #[serde(rename = "createdAt")]
    pub created_at: DateTime<Utc>,

    #[serde(rename = "deviceId")]
    #[allow(dead_code)]
    pub device_id: String,

    #[serde(rename = "originalPath")]
    #[allow(dead_code)]
    pub original_path: String,
}

#[derive(Serialize, Debug)]
struct DeleteAssetsRequest<'a> {
    ids: Vec<&'a str>,
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Cannot parse the URL path: {0}")]
    Path(#[from] url::ParseError),

    #[error("Error when request to Immich API: {0}")]
    Network(#[from] reqwest::Error),

    #[error("Asset is not found on Immich instance")]
    AssetNotFound,
}

pub struct Service {
    client: Client,
    host: String,
    api_key: String,
}

impl Service {
    pub fn new(host: &str, api_key: &str) -> Self {
        let client = Client::new();
        Service {
            client,
            host: host.to_string(),
            api_key: api_key.to_string(),
        }
    }

    pub async fn get_asset_by_checksum(&self, checksum: &str) -> Result<Asset, Error> {
        let mut url = Url::parse(&self.host)?;
        url.set_path("/api/search/metadata");

        let request = SearchMetadataRequest { checksum };

        let response = self
            .client
            .post(url)
            .header("x-api-key", &self.api_key)
            .json(&request)
            .send()
            .await?
            .error_for_status()?;

        let response_json: SearchMetadataResponse = response.json().await?;

        if let Some(asset) = response_json.assets.items.first() {
            Ok(asset.clone())
        } else {
            Err(Error::AssetNotFound)
        }
    }

    pub async fn delete_assets(&self, ids: Vec<&str>) -> Result<(), Error> {
        let mut url = Url::parse(&self.host)?;
        url.set_path("/api/assets");

        let request = DeleteAssetsRequest { ids };

        self.client
            .delete(url)
            .header("x-api-key", &self.api_key)
            .json(&request)
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }
}
