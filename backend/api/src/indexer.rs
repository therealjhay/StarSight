//! Background indexer that polls the Stellar Soroban RPC for contract events
//! and upserts parsed data into PostgreSQL. New prediction events are also
//! broadcast to active WebSocket connections.

use base64::{engine::general_purpose::STANDARD, Engine as _};
use serde::{Deserialize, Serialize};
use stellar_xdr::curr::{ReadXdr, ScVal};

use crate::config::Config;
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
#[allow(dead_code)]
struct SorobanEvent {
    #[serde(rename = "type")]
    event_type: String,
    ledger: u64,
    #[serde(rename = "contractId")]
    contract_id: String,
    topic: Vec<String>,
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

pub async fn run(state: AppState, config: Config) {
    let client = reqwest::Client::new();

    let mut cursor_ledger = match fetch_latest_ledger(&client, &config.stellar_rpc_url).await {
        Ok(seq) => {
            let start = seq.saturating_sub(100);
            tracing::info!("Indexer starting from ledger {} (latest: {})", start, seq);
            start
        }
        Err(e) => {
            tracing::error!("Failed to fetch latest ledger: {}. Starting from 0.", e);
            0
        }
    };

    loop {
        match poll_events(&client, &config, &state, cursor_ledger).await {
            Ok(new_cursor) => {
                if new_cursor > cursor_ledger {
                    tracing::info!("Indexer advanced cursor from {} to {}", cursor_ledger, new_cursor);
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
    resp.result.map(|r| r.sequence).ok_or_else(|| "No result".into())
}

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

    let result = resp.result.ok_or("No result")?;
    let latest_ledger = result.latest_ledger.unwrap_or(start_ledger);

    if let Some(events) = result.events {
        for event in &events {
            if let Err(e) = process_event(event, config, state).await {
                tracing::error!("Failed to process event at ledger {}: {}", event.ledger, e);
            }
        }
    }

    Ok(latest_ledger.saturating_add(1))
}

async fn process_event(
    event: &SorobanEvent,
    config: &Config,
    state: &AppState,
) -> Result<(), Box<dyn std::error::Error>> {
    let contract_id = &event.contract_id;

    if contract_id == &config.asset_registry_contract_id {
        process_asset_event(event, state).await
    } else if contract_id == &config.prediction_market_contract_id {
        process_prediction_event(event, state).await
    } else if contract_id == &config.reputation_contract_id {
        process_reputation_event(event, state).await
    } else {
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// XDR Decoding Helpers
// ---------------------------------------------------------------------------

fn decode_b64(b64: &str) -> Result<ScVal, Box<dyn std::error::Error>> {
    let bytes = STANDARD.decode(b64)?;
    Ok(ScVal::from_xdr(bytes, stellar_xdr::curr::Limits::none())?)
}

fn map_get<'a>(val: &'a ScVal, key: &str) -> Option<&'a ScVal> {
    if let ScVal::Map(Some(m)) = val {
        for entry in m.iter() {
            if let ScVal::Symbol(sym) = &entry.key {
                if sym.to_string() == key {
                    return Some(&entry.val);
                }
            }
        }
    }
    None
}

fn ext_str(val: &ScVal) -> Option<String> {
    match val {
        ScVal::String(s) => String::from_utf8(s.to_vec()).ok(),
        ScVal::Symbol(s) => Some(s.to_string()),
        ScVal::Address(addr) => match addr {
            stellar_xdr::curr::ScAddress::Account(acc) => {
                Some(format!("{:?}", acc))
            }
            stellar_xdr::curr::ScAddress::Contract(hash) => {
                let arr: [u8; 32] = hash.0.to_vec().try_into().unwrap_or([0; 32]);
                Some(stellar_strkey::Contract(arr).to_string().as_str().to_string())
            }
        },
        ScVal::Vec(Some(v)) => v.get(0).and_then(ext_str), // Handle enums
        ScVal::Bytes(b) => {
             if b.len() == 32 {
                 let arr: [u8; 32] = b.to_vec().try_into().unwrap();
                 Some(stellar_strkey::Contract(arr).to_string().as_str().to_string())
             } else {
                 Some(String::from_utf8_lossy(&b.to_vec()).to_string())
             }
        }
        _ => None,
    }
}

fn ext_i64(val: &ScVal) -> Option<i64> {
    match val {
        ScVal::I64(v) => Some(*v),
        ScVal::U64(v) => Some(*v as i64),
        ScVal::I32(v) => Some(*v as i64),
        ScVal::U32(v) => Some(*v as i64),
        _ => None,
    }
}

fn ext_bool(val: &ScVal) -> Option<bool> {
    if let ScVal::Bool(b) = val {
        Some(*b)
    } else {
        None
    }
}

// ---------------------------------------------------------------------------
// Event parsers
// ---------------------------------------------------------------------------

async fn process_asset_event(
    event: &SorobanEvent,
    state: &AppState,
) -> Result<(), Box<dyn std::error::Error>> {
    if event.topic.is_empty() { return Ok(()); }
    let topic0 = decode_b64(&event.topic[0])?;
    let func_name = ext_str(&topic0).unwrap_or_default();

    if func_name == "register_asset" || func_name == "deactivate_asset" {
        let val = decode_b64(&event.value)?;
        
        let asset = Asset {
            id: map_get(&val, "id").and_then(ext_str).unwrap_or_default(),
            name: map_get(&val, "name").and_then(ext_str).unwrap_or_default(),
            issuer: map_get(&val, "issuer").and_then(ext_str).unwrap_or_default(),
            asset_type: map_get(&val, "asset_type").and_then(ext_str).unwrap_or_default(),
            stellar_asset_contract: map_get(&val, "stellar_asset_contract").and_then(ext_str).unwrap_or_default(),
            registered_at: map_get(&val, "registered_at").and_then(ext_i64).unwrap_or(0),
            is_active: map_get(&val, "is_active").and_then(ext_bool).unwrap_or(true),
        };

        if !asset.id.is_empty() {
            crate::db::upsert_asset(&state.pool, &asset).await?;
            tracing::info!("Upserted asset: {}", asset.id);
        }
    }

    Ok(())
}

async fn process_prediction_event(
    event: &SorobanEvent,
    state: &AppState,
) -> Result<(), Box<dyn std::error::Error>> {
    if event.topic.is_empty() { return Ok(()); }
    let topic0 = decode_b64(&event.topic[0])?;
    let func_name = ext_str(&topic0).unwrap_or_default();

    if func_name == "submit_prediction" {
        let val = decode_b64(&event.value)?;

        let prediction = Prediction {
            id: map_get(&val, "id").and_then(ext_i64).unwrap_or(0),
            agent: map_get(&val, "agent").and_then(ext_str).unwrap_or_default(),
            asset_id: map_get(&val, "asset_id").and_then(ext_str).unwrap_or_default(),
            prediction_type: map_get(&val, "prediction_type").and_then(ext_str).unwrap_or_default(),
            value: map_get(&val, "value").and_then(ext_i64).unwrap_or(0),
            confidence: map_get(&val, "confidence").and_then(ext_i64).unwrap_or(0) as i32,
            submitted_at: map_get(&val, "submitted_at").and_then(ext_i64).unwrap_or(0),
            resolution_ledger: map_get(&val, "resolution_ledger").and_then(ext_i64).unwrap_or(0),
            status: map_get(&val, "status").and_then(ext_str).unwrap_or_else(|| "Pending".to_string()),
            resolved_value: map_get(&val, "resolved_value").and_then(ext_i64),
        };

        crate::db::upsert_prediction(&state.pool, &prediction).await?;
        tracing::info!("Upserted prediction: {}", prediction.id);

        // Broadcast new predictions to WebSocket clients.
        if let Ok(json) = serde_json::to_string(&prediction) {
            let _ = state.tx.send(json); // Ignore error if no receivers.
        }
    } else if func_name == "resolve_prediction" {
        let val = decode_b64(&event.value)?;
        let id = map_get(&val, "id").and_then(ext_i64).unwrap_or(0);
        
        if id > 0 {
            if let Some(mut existing) = crate::db::get_prediction_by_id(&state.pool, id).await? {
                existing.status = map_get(&val, "status").and_then(ext_str).unwrap_or_else(|| "Resolved".to_string());
                existing.resolved_value = map_get(&val, "resolved_value").and_then(ext_i64);
                crate::db::upsert_prediction(&state.pool, &existing).await?;
                tracing::info!("Resolved prediction: {}", id);
            }
        }
    }

    Ok(())
}

async fn process_reputation_event(
    event: &SorobanEvent,
    state: &AppState,
) -> Result<(), Box<dyn std::error::Error>> {
    if event.topic.is_empty() { return Ok(()); }
    let topic0 = decode_b64(&event.topic[0])?;
    let func_name = ext_str(&topic0).unwrap_or_default();

    if func_name == "score_prediction" {
        let val = decode_b64(&event.value)?;

        let score = AgentScore {
            agent: map_get(&val, "agent").and_then(ext_str).unwrap_or_default(),
            total_predictions: map_get(&val, "total_predictions").and_then(ext_i64).unwrap_or(0) as i32,
            correct_predictions: map_get(&val, "correct_predictions").and_then(ext_i64).unwrap_or(0) as i32,
            accuracy_bps: map_get(&val, "accuracy_bps").and_then(ext_i64).unwrap_or(0) as i32,
            streak: map_get(&val, "streak").and_then(ext_i64).unwrap_or(0) as i32,
            last_scored_at: map_get(&val, "last_scored_at").and_then(ext_i64).unwrap_or(0),
        };

        if !score.agent.is_empty() {
            crate::db::upsert_agent_score(&state.pool, &score).await?;
            tracing::info!("Upserted reputation for: {}", score.agent);
        }
    }

    Ok(())
}
