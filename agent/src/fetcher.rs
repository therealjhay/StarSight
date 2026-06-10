use crate::config::Config;
use crate::models::AssetMarketData;

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

/// Errors that can occur when fetching market data from the backend API.
#[derive(Debug)]
pub enum FetchError {
    /// HTTP request failed (network error, timeout, etc.).
    Http(reqwest::Error),
    /// Server returned a non-success status code.
    Status(reqwest::StatusCode, String),
    /// Response body could not be deserialized.
    Deserialize(reqwest::Error),
}

impl std::fmt::Display for FetchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FetchError::Http(e) => write!(f, "HTTP request failed: {}", e),
            FetchError::Status(code, body) => {
                write!(f, "API returned status {}: {}", code, body)
            }
            FetchError::Deserialize(e) => write!(f, "Failed to deserialize response: {}", e),
        }
    }
}

impl std::error::Error for FetchError {}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Fetches pre-aggregated market data for all active assets from the backend.
///
/// Calls `GET {api_base_url}/assets/market-data` and deserializes the JSON
/// response into a list of [`AssetMarketData`].
pub async fn fetch_market_data(
    client: &reqwest::Client,
    config: &Config,
) -> Result<Vec<AssetMarketData>, FetchError> {
    let url = format!("{}/assets/market-data", config.api_base_url);

    let response = client.get(&url).send().await.map_err(FetchError::Http)?;

    let status = response.status();
    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        return Err(FetchError::Status(status, body));
    }

    let assets: Vec<AssetMarketData> = response
        .json()
        .await
        .map_err(FetchError::Deserialize)?;

    Ok(assets)
}
