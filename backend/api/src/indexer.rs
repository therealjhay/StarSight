//! Background indexer that polls the Stellar Soroban RPC for contract events
//! and upserts parsed data into PostgreSQL. New prediction events are also
//! broadcast to active WebSocket connections.

use serde::{Deserialize, Serialize};

use crate::config::Config;
// These model types will be used once XDR event parsing is implemented.
#[allow(unused_imports)]
use crate::models::{AgentScore, Asset, Prediction};
use crate::AppState;

/// How often the indexer polls for new events (in seconds).
const POLL_INTERVAL_SECS: u64 = 10;

/// Maximum number of events to request per RPC call.
const EVENT_LIMIT: u32 = 100;

// ---------------------------------------------------------------------------
// Soroban RPC JSON-RPC request/response types
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
struct JsonRpcRequest<T: Serialize> {
    jsonrpc: &'static str,
    id: u64,
    method: &'static str,
    params: T,
}

#[derive(Debug, Serialize)]
struct GetEventsParams {
    #[serde(rename = "startLedger")]
    start_ledger: u64,
    filters: Vec<EventFilter>,
    pagination: Pagination,
}

#[derive(Debug, Serialize)]
struct EventFilter {
    #[serde(rename = "type")]
    event_type: String,
    #[serde(rename = "contractIds")]
    contract_ids: Vec<String>,
}

#[derive(Debug, Serialize)]
struct Pagination {
    limit: u32,
}

#[derive(Debug, Deserialize)]
struct JsonRpcResponse<T> {
    result: Option<T>,
    error: Option<JsonRpcError>,
}

#[derive(Debug, Deserialize)]
struct JsonRpcError {
    code: i64,
    message: String,
}

#[derive(Debug, Deserialize)]
struct GetEventsResult {
    events: Option<Vec<SorobanEvent>>,
    #[serde(rename = "latestLedger")]
    latest_ledger: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct SorobanEvent {
    /// The type of event (contract, system, diagnostic).
    #[serde(rename = "type")]
    event_type: String,
    /// Ledger sequence the event was emitted in.
    ledger: u64,
    /// Contract ID that emitted the event.
    #[serde(rename = "contractId")]
    contract_id: String,
    /// XDR-encoded topic segments (base64).
    topic: Vec<String>,
    /// XDR-encoded event value (base64).
    value: String,
}

#[derive(Debug, Serialize)]
struct GetLatestLedgerParams {}

#[derive(Debug, Deserialize)]
struct GetLatestLedgerResult {
    sequence: u64,
}

// ---------------------------------------------------------------------------
// Indexer main loop
// ---------------------------------------------------------------------------

/// Runs the event indexer loop. This function never returns under normal operation.
/// On errors, it logs and retries after the poll interval.
pub async fn run(state: AppState, config: Config) {
    let client = reqwest::Client::new();

    // Determine the starting ledger by fetching the latest.
    let mut cursor_ledger = match fetch_latest_ledger(&client, &config.stellar_rpc_url).await {
        Ok(seq) => {
            // Start a few ledgers back to catch any events we might have missed.
            let start = seq.saturating_sub(100);
            tracing::info!(
                "Indexer starting from ledger {} (latest: {})",
                start,
                seq
            );
            start
        }
        Err(e) => {
            tracing::error!("Failed to fetch latest ledger on startup: {}. Starting from 0.", e);
            0
        }
    };

    loop {
        match poll_events(&client, &config, &state, cursor_ledger).await {
            Ok(new_cursor) => {
                if new_cursor > cursor_ledger {
                    tracing::info!(
                        "Indexer advanced cursor from ledger {} to {}",
                        cursor_ledger,
                        new_cursor
                    );
                    cursor_ledger = new_cursor;
                }
            }
            Err(e) => {
                tracing::error!("Indexer poll error: {}. Will retry.", e);
            }
        }

        tokio::time::sleep(tokio::time::Duration::from_secs(POLL_INTERVAL_SECS)).await;
    }
}

/// Fetches the latest ledger sequence from the Soroban RPC.
async fn fetch_latest_ledger(
    client: &reqwest::Client,
    rpc_url: &str,
) -> Result<u64, Box<dyn std::error::Error>> {
    let req = JsonRpcRequest {
        jsonrpc: "2.0",
        id: 1,
        method: "getLatestLedger",
        params: GetLatestLedgerParams {},
    };

    let resp: JsonRpcResponse<GetLatestLedgerResult> =
        client.post(rpc_url).json(&req).send().await?.json().await?;

    if let Some(err) = resp.error {
        return Err(format!("RPC error {}: {}", err.code, err.message).into());
    }

    resp.result
        .map(|r| r.sequence)
        .ok_or_else(|| "No result in getLatestLedger response".into())
}

/// Polls the Soroban RPC for events from all four contracts starting at `start_ledger`.
/// Returns the new cursor ledger to use for the next poll.
async fn poll_events(
    client: &reqwest::Client,
    config: &Config,
    state: &AppState,
    start_ledger: u64,
) -> Result<u64, Box<dyn std::error::Error>> {
    let contract_ids = vec![
        config.asset_registry_contract_id.clone(),
        config.prediction_market_contract_id.clone(),
        config.reputation_contract_id.clone(),
        config.rewards_contract_id.clone(),
    ];

    let params = GetEventsParams {
        start_ledger,
        filters: vec![EventFilter {
            event_type: "contract".to_string(),
            contract_ids,
        }],
        pagination: Pagination { limit: EVENT_LIMIT },
    };

    let req = JsonRpcRequest {
        jsonrpc: "2.0",
        id: 1,
        method: "getEvents",
        params,
    };

    let resp: JsonRpcResponse<GetEventsResult> =
        client.post(&config.stellar_rpc_url).json(&req).send().await?.json().await?;

    if let Some(err) = resp.error {
        return Err(format!("RPC error {}: {}", err.code, err.message).into());
    }

    let result = resp.result.ok_or("No result in getEvents response")?;
    let latest_ledger = result.latest_ledger.unwrap_or(start_ledger);

    if let Some(events) = result.events {
        if !events.is_empty() {
            tracing::info!("Indexer received {} events", events.len());
        }

        for event in &events {
            if let Err(e) = process_event(event, config, state).await {
                tracing::error!(
                    "Failed to process event from contract {} at ledger {}: {}",
                    event.contract_id,
                    event.ledger,
                    e
                );
                // Continue processing remaining events.
            }
        }
    }

    // Advance past the latest ledger we've processed.
    Ok(latest_ledger.saturating_add(1))
}

/// Processes a single Soroban contract event by routing it to the correct
/// handler based on which contract emitted it.
async fn process_event(
    event: &SorobanEvent,
    config: &Config,
    state: &AppState,
) -> Result<(), Box<dyn std::error::Error>> {
    // Determine which contract emitted the event.
    let contract_id = &event.contract_id;

    if contract_id == &config.asset_registry_contract_id {
        process_asset_event(event, state).await
    } else if contract_id == &config.prediction_market_contract_id {
        process_prediction_event(event, state).await
    } else if contract_id == &config.reputation_contract_id {
        process_reputation_event(event, state).await
    } else if contract_id == &config.rewards_contract_id {
        // Rewards events don't require indexing into the DB currently.
        tracing::info!("Rewards contract event at ledger {}", event.ledger);
        Ok(())
    } else {
        tracing::warn!("Unknown contract ID in event: {}", contract_id);
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Event parsers — stubs with TODO markers
//
// Each function receives a SorobanEvent whose `topic` and `value` fields are
// base64-encoded XDR. The exact XDR types to decode are:
//   - topic[0]: ScVal::Symbol — the function/event name
//   - topic[1..]: ScVal variants — indexed parameters
//   - value: ScVal — the event body
//
// Use `stellar_xdr::curr::{ReadXdr, ScVal}` to decode:
//   let bytes = base64::decode(&event.topic[0])?;
//   let sc_val = ScVal::from_xdr(bytes)?;
// ---------------------------------------------------------------------------

async fn process_asset_event(
    event: &SorobanEvent,
    state: &AppState,
) -> Result<(), Box<dyn std::error::Error>> {
    // TODO: Decode event.topic[0] as ScVal::Symbol to determine the function name.
    //       Expected symbols: "register_asset", "deactivate_asset"
    //
    // TODO: Decode event.value as the Asset struct XDR (ScVal::Map with fields:
    //       id, name, issuer, asset_type, stellar_asset_contract, registered_at, is_active).
    //
    // For now, log the raw event for debugging.
    tracing::info!(
        "Asset registry event at ledger {} — topic count: {}, value len: {} bytes",
        event.ledger,
        event.topic.len(),
        event.value.len()
    );

    // Stub: When XDR parsing is implemented, upsert the decoded Asset:
    // let asset = parse_asset_from_xdr(&event.topic, &event.value)?;
    // crate::db::upsert_asset(&state.pool, &asset).await?;

    let _ = state; // suppress unused warning until parsing is implemented
    Ok(())
}

async fn process_prediction_event(
    event: &SorobanEvent,
    state: &AppState,
) -> Result<(), Box<dyn std::error::Error>> {
    // TODO: Decode event.topic[0] as ScVal::Symbol to determine the function name.
    //       Expected symbols: "submit_prediction", "resolve_prediction"
    //
    // TODO: For "submit_prediction", decode event.value as the Prediction struct XDR
    //       (ScVal::Map with fields: id, agent, asset_id, prediction_type, value,
    //       confidence, submitted_at, resolution_ledger, status, resolved_value).
    //
    // TODO: For "resolve_prediction", decode event.value to get the prediction ID
    //       and actual_value, then update the existing row.
    tracing::info!(
        "Prediction market event at ledger {} — topic count: {}, value len: {} bytes",
        event.ledger,
        event.topic.len(),
        event.value.len()
    );

    // Stub: When XDR parsing is implemented:
    // let prediction = parse_prediction_from_xdr(&event.topic, &event.value)?;
    // crate::db::upsert_prediction(&state.pool, &prediction).await?;
    //
    // // Broadcast new predictions to WebSocket clients.
    // if is_submit_event {
    //     let json = serde_json::to_string(&prediction)?;
    //     let _ = state.tx.send(json); // Ignore error if no receivers.
    // }

    let _ = state; // suppress unused warning until parsing is implemented
    Ok(())
}

async fn process_reputation_event(
    event: &SorobanEvent,
    state: &AppState,
) -> Result<(), Box<dyn std::error::Error>> {
    // TODO: Decode event.topic[0] as ScVal::Symbol to determine the function name.
    //       Expected symbol: "score_prediction"
    //
    // TODO: Decode event.value to extract the updated ReputationScore struct
    //       (ScVal::Map with fields: agent, total_predictions, correct_predictions,
    //       accuracy_bps, streak, last_scored_at).
    tracing::info!(
        "Reputation event at ledger {} — topic count: {}, value len: {} bytes",
        event.ledger,
        event.topic.len(),
        event.value.len()
    );

    // Stub: When XDR parsing is implemented:
    // let score = parse_agent_score_from_xdr(&event.topic, &event.value)?;
    // crate::db::upsert_agent_score(&state.pool, &score).await?;

    let _ = state; // suppress unused warning until parsing is implemented
    Ok(())
}
