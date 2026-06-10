use std::collections::HashMap;
use std::time::Duration;

mod analyzer;
mod config;
mod fetcher;
mod models;
mod submitter;

use config::Config;
use models::SubmittedPrediction;

#[tokio::main]
async fn main() {
    // Load .env from the workspace root.
    dotenv::dotenv().ok();

    // Initialize structured logging with tracing.
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "starsight_agent=info".into()),
        )
        .init();

    // Parse typed configuration from environment variables.
    let config = Config::from_env();

    tracing::info!(
        poll_interval_secs = config.poll_interval_secs,
        rpc_url = %config.stellar_rpc_url,
        contract_id = %config.prediction_market_contract_id,
        "StarSight agent starting"
    );

    // Shared HTTP client for all outbound requests (API + Claude + RPC).
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .expect("Failed to build HTTP client");

    // In-memory session map: prediction_id → SubmittedPrediction.
    let mut session_map: HashMap<u64, SubmittedPrediction> = HashMap::new();
    let mut cycle: u64 = 0;

    let interval = Duration::from_secs(config.poll_interval_secs);

    loop {
        // Wait for the next cycle or a shutdown signal.
        tokio::select! {
            _ = tokio::time::sleep(interval) => {
                // Time for a new cycle.
            }
            _ = tokio::signal::ctrl_c() => {
                tracing::info!("Agent shutting down cleanly");
                std::process::exit(0);
            }
        }

        cycle += 1;
        run_cycle(&client, &config, &mut session_map, cycle).await;
    }
}

/// Executes a single poll cycle: fetch → analyze → submit → log.
async fn run_cycle(
    client: &reqwest::Client,
    config: &Config,
    session_map: &mut HashMap<u64, SubmittedPrediction>,
    cycle: u64,
) {
    // --- FETCH ---
    let assets = match fetcher::fetch_market_data(client, config).await {
        Ok(assets) => assets,
        Err(e) => {
            tracing::error!(cycle = cycle, error = %e, "Failed to fetch market data");
            tracing::info!(
                "[CYCLE {}] Assets analyzed: 0 | Predictions submitted: 0 | Failed: 0 (fetch error)",
                cycle
            );
            return;
        }
    };

    let total_assets = assets.len();
    let mut submitted = 0u64;
    let mut failed = 0u64;

    for asset in &assets {
        // --- ANALYZE ---
        let prediction = match analyzer::analyze(client, config, asset).await {
            Ok(pred) => {
                tracing::info!(
                    asset_id = %asset.asset_id,
                    prediction_type = %pred.prediction_type,
                    value = pred.value,
                    confidence = pred.confidence,
                    reasoning = %pred.reasoning,
                    "Prediction generated"
                );

                // Skip predictions with zero confidence (guardrail).
                if pred.confidence == 0 {
                    tracing::warn!(
                        asset_id = %asset.asset_id,
                        "Skipping prediction with zero confidence"
                    );
                    failed += 1;
                    continue;
                }

                pred
            }
            Err(e) => {
                tracing::error!(
                    asset_id = %asset.asset_id,
                    error = %e,
                    "Failed to analyze asset — skipping"
                );
                failed += 1;
                continue;
            }
        };

        // --- SUBMIT ---
        match submitter::submit_prediction(client, config, &asset.asset_id, &prediction).await {
            Ok(prediction_id) => {
                let record = SubmittedPrediction {
                    prediction_id,
                    asset_id: asset.asset_id.clone(),
                    prediction_type: prediction.prediction_type.clone(),
                    value: prediction.value,
                    confidence: prediction.confidence,
                    submitted_at: chrono::Utc::now(),
                };
                session_map.insert(prediction_id, record);
                submitted += 1;
            }
            Err(e) => {
                tracing::error!(
                    asset_id = %asset.asset_id,
                    error = %e,
                    "Failed to submit prediction — continuing to next asset"
                );
                failed += 1;
            }
        }
    }

    // --- LOG ---
    tracing::info!(
        "[CYCLE {}] Assets analyzed: {} | Predictions submitted: {} | Failed: {}",
        cycle,
        total_assets,
        submitted,
        failed
    );

    if !session_map.is_empty() {
        tracing::debug!(
            session_predictions = session_map.len(),
            "Session map updated"
        );
    }
}
