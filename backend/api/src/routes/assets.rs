use std::sync::Arc;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Json, Router};
use serde::Serialize;

use crate::models::ErrorResponse;
use crate::AppState;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/assets", get(list_assets))
        .route("/assets/{id}", get(get_asset))
        .route("/assets/market-data", get(get_market_data))
}

// ---------------------------------------------------------------------------
// Market data stub types (consumed by the StarSight agent)
// ---------------------------------------------------------------------------

/// Pre-aggregated market data for a single RWA asset.
/// This is a temporary stub — replace with real oracle/price feed integration.
#[derive(Debug, Clone, Serialize)]
pub struct AssetMarketDataResponse {
    pub asset_id: String,
    pub name: String,
    pub asset_type: String,
    /// Current price scaled by 1_000_000.
    pub current_price: i64,
    /// 24-hour price change as a percentage.
    pub price_change_24h: f64,
    /// Current annualized yield in basis points.
    pub current_yield: i64,
    /// 30-day average yield in basis points.
    pub avg_yield_30d: i64,
    /// 30-day volatility as a percentage.
    pub volatility_30d: f64,
    /// Total market cap scaled by 1_000_000.
    pub market_cap: i64,
    /// ISO 8601 timestamp of last data update.
    pub last_updated: String,
}

/// GET /assets/market-data — Returns hardcoded plausible market data for all
/// active RWA assets. This is a temporary stub for the StarSight agent to
/// consume until real oracle integration is built.
async fn get_market_data(
    State(_state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let now = chrono::Utc::now().to_rfc3339();

    let data = vec![
        AssetMarketDataResponse {
            asset_id: "REIT01".to_string(),
            name: "US Commercial REIT Fund".to_string(),
            asset_type: "RealEstate".to_string(),
            current_price: 1_050_000_000,   // $1,050.00
            price_change_24h: -2.3,
            current_yield: 540,              // 5.40% annualized
            avg_yield_30d: 520,
            volatility_30d: 12.5,
            market_cap: 500_000_000_000_000, // $500B
            last_updated: now.clone(),
        },
        AssetMarketDataResponse {
            asset_id: "TBOND1".to_string(),
            name: "Tokenized US 10Y Treasury".to_string(),
            asset_type: "Treasury".to_string(),
            current_price: 985_500_000,     // $985.50
            price_change_24h: 0.15,
            current_yield: 425,              // 4.25% annualized
            avg_yield_30d: 430,
            volatility_30d: 3.2,
            market_cap: 1_200_000_000_000_000, // $1.2T
            last_updated: now.clone(),
        },
        AssetMarketDataResponse {
            asset_id: "GOLD01".to_string(),
            name: "Gold-Backed Commodity Token".to_string(),
            asset_type: "Commodity".to_string(),
            current_price: 2_340_000_000,   // $2,340.00
            price_change_24h: 1.8,
            current_yield: 0,                // Gold has no yield
            avg_yield_30d: 0,
            volatility_30d: 8.7,
            market_cap: 200_000_000_000_000, // $200B
            last_updated: now,
        },
    ];

    (StatusCode::OK, Json(data))
}

/// GET /assets — Returns all registered assets.
async fn list_assets(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    match crate::db::get_all_assets(&state.pool).await {
        Ok(assets) => (StatusCode::OK, Json(serde_json::to_value(assets).unwrap())),
        Err(e) => {
            tracing::error!("Failed to fetch assets: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::to_value(ErrorResponse {
                    error: "Failed to fetch assets".into(),
                }).unwrap()),
            )
        }
    }
}

/// GET /assets/:id — Returns a single asset by ID.
async fn get_asset(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match crate::db::get_asset_by_id(&state.pool, &id).await {
        Ok(Some(asset)) => (StatusCode::OK, Json(serde_json::to_value(asset).unwrap())),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::to_value(ErrorResponse {
                error: format!("Asset '{}' not found", id),
            }).unwrap()),
        ),
        Err(e) => {
            tracing::error!("Failed to fetch asset {}: {}", id, e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::to_value(ErrorResponse {
                    error: "Failed to fetch asset".into(),
                }).unwrap()),
            )
        }
    }
}
