use serde::{Deserialize, Serialize};

use crate::config::Config;
use crate::models::RawPrediction;

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

/// Errors that can occur when submitting a prediction to the Soroban contract.
#[derive(Debug)]
#[allow(dead_code)]
pub enum SubmitError {
    /// Failed to decode the deployer secret key.
    KeyDecode(String),
    /// HTTP request to the Stellar RPC failed.
    Http(reqwest::Error),
    /// Stellar RPC returned an error response.
    RpcError(String),
    /// Transaction simulation failed.
    SimulationFailed(String),
    /// Transaction submission failed.
    TransactionFailed(String),
    /// Failed to parse the transaction result.
    ParseResult(String),
}

impl std::fmt::Display for SubmitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SubmitError::KeyDecode(e) => write!(f, "Key decode error: {}", e),
            SubmitError::Http(e) => write!(f, "RPC HTTP error: {}", e),
            SubmitError::RpcError(e) => write!(f, "RPC error: {}", e),
            SubmitError::SimulationFailed(e) => write!(f, "Simulation failed: {}", e),
            SubmitError::TransactionFailed(e) => write!(f, "Transaction failed: {}", e),
            SubmitError::ParseResult(e) => write!(f, "Result parse error: {}", e),
        }
    }
}

impl std::error::Error for SubmitError {}

// ---------------------------------------------------------------------------
// Soroban RPC JSON-RPC types
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    id: u64,
    method: String,
    params: serde_json::Value,
}

#[derive(Debug, Deserialize)]
struct JsonRpcResponse {
    result: Option<serde_json::Value>,
    error: Option<JsonRpcError>,
}

#[derive(Debug, Deserialize)]
struct JsonRpcError {
    code: Option<i64>,
    message: Option<String>,
}

impl std::fmt::Display for JsonRpcError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "code={}, message={}",
            self.code.unwrap_or(0),
            self.message.as_deref().unwrap_or("unknown")
        )
    }
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Submits a prediction to the prediction-market Soroban contract.
///
/// This function uses the Stellar CLI (`stellar contract invoke`) to build,
/// sign, and submit the transaction. This is a pragmatic approach that avoids
/// the complexity of manual XDR construction while the Rust Stellar SDK
/// ecosystem matures.
///
/// # Arguments
/// * `client` — Shared HTTP client (unused in CLI mode, kept for API consistency).
/// * `config` — Agent configuration with RPC URL, keys, and contract ID.
/// * `asset_id` — On-chain asset identifier (e.g. "REIT01").
/// * `prediction` — The parsed prediction from Claude.
///
/// # Returns
/// The on-chain prediction ID on success.
///
/// # TODO
/// Replace CLI shelling with native XDR construction using `stellar-xdr` +
/// `ed25519-dalek` once the Rust SDK provides a stable off-chain transaction
/// builder. The manual approach requires: building `InvokeHostFunctionOp`,
/// simulating via RPC, assembling the footprint, signing the envelope, and
/// submitting — which is ~300 lines of brittle XDR plumbing.
pub async fn submit_prediction(
    _client: &reqwest::Client,
    config: &Config,
    asset_id: &str,
    prediction: &RawPrediction,
) -> Result<u64, SubmitError> {
    // Map PredictionType to the contract's enum variant index.
    let prediction_type_val = match prediction.prediction_type {
        crate::models::PredictionType::PriceTarget => "PriceTarget",
        crate::models::PredictionType::YieldForecast => "YieldForecast",
        crate::models::PredictionType::RiskScore => "RiskScore",
    };

    // The value needs to be i128 on-chain, passed as a string to the CLI.
    let value_i128 = prediction.value as i128;

    // Resolution ledger: current ledger + ~24 hours (17,280 ledgers at 5s each).
    // We fetch the current ledger from the RPC first.
    let current_ledger = get_latest_ledger(_client, config).await?;
    let resolution_ledger = current_ledger + 17_280;

    tracing::debug!(
        asset_id = %asset_id,
        prediction_type = %prediction_type_val,
        value = %value_i128,
        confidence = %prediction.confidence,
        resolution_ledger = %resolution_ledger,
        "Submitting prediction via Stellar CLI"
    );

    // Shell out to `stellar contract invoke` to submit the prediction.
    // This avoids the complexity of manual XDR construction.
    let output = tokio::process::Command::new("stellar")
        .args([
            "contract",
            "invoke",
            "--id",
            &config.prediction_market_contract_id,
            "--source-account",
            &config.deployer_secret_key,
            "--rpc-url",
            &config.stellar_rpc_url,
            "--network-passphrase",
            &config.network_passphrase,
            "--send=yes",
            "--",
            "submit_prediction",
            // Agent address is derived from the source key by the CLI.
            "--agent",
            &source_account_address(config)?,
            "--asset_id",
            asset_id,
            "--prediction_type",
            prediction_type_val,
            "--value",
            &value_i128.to_string(),
            "--confidence",
            &prediction.confidence.to_string(),
            "--resolution_ledger",
            &resolution_ledger.to_string(),
        ])
        .output()
        .await
        .map_err(|e| SubmitError::TransactionFailed(format!("Failed to spawn stellar CLI: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        return Err(SubmitError::TransactionFailed(format!(
            "CLI exited with status {}.\nstdout: {}\nstderr: {}",
            output.status, stdout, stderr
        )));
    }

    // Parse the prediction ID from stdout. The CLI returns the function's
    // return value as a raw string (e.g. "1" or "42").
    let stdout = String::from_utf8_lossy(&output.stdout);
    let prediction_id: u64 = stdout
        .trim()
        .parse()
        .map_err(|e| {
            SubmitError::ParseResult(format!(
                "Failed to parse prediction_id from CLI output '{}': {}",
                stdout.trim(),
                e
            ))
        })?;

    tracing::info!(
        prediction_id = prediction_id,
        asset_id = %asset_id,
        "Prediction submitted successfully"
    );

    Ok(prediction_id)
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Derives the public Stellar address (G...) from the secret key (S...).
fn source_account_address(config: &Config) -> Result<String, SubmitError> {
    use ed25519_dalek::SigningKey;
    use stellar_strkey::{ed25519::PublicKey as StrkeyPublicKey, Strkey};

    let secret = config.deployer_secret_key.trim();
    let strkey = Strkey::from_string(secret)
        .map_err(|e| SubmitError::KeyDecode(format!("Invalid secret key: {}", e)))?;

    let seed = match strkey {
        Strkey::PrivateKeyEd25519(s) => s.0,
        _ => {
            return Err(SubmitError::KeyDecode(
                "Expected an Ed25519 private key (S...)".to_string(),
            ))
        }
    };

    let signing_key = SigningKey::from_bytes(&seed);
    let public_key = signing_key.verifying_key();

    let strkey_pub = StrkeyPublicKey(public_key.to_bytes());
    Ok(strkey_pub.to_string())
}

/// Fetches the latest ledger sequence from the Soroban RPC.
async fn get_latest_ledger(
    client: &reqwest::Client,
    config: &Config,
) -> Result<u64, SubmitError> {
    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: 1,
        method: "getLatestLedger".to_string(),
        params: serde_json::json!({}),
    };

    let response: JsonRpcResponse = client
        .post(&config.stellar_rpc_url)
        .json(&request)
        .send()
        .await
        .map_err(SubmitError::Http)?
        .json()
        .await
        .map_err(SubmitError::Http)?;

    if let Some(err) = response.error {
        return Err(SubmitError::RpcError(err.to_string()));
    }

    let result = response
        .result
        .ok_or_else(|| SubmitError::RpcError("No result in getLatestLedger response".to_string()))?;

    let sequence = result
        .get("sequence")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| {
            SubmitError::ParseResult(format!(
                "Missing 'sequence' in getLatestLedger result: {}",
                result
            ))
        })?;

    Ok(sequence)
}
