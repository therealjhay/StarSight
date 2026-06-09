use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Mirror of the on-chain Asset struct from the `asset-registry` contract.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Asset {
    pub id: String,
    pub name: String,
    pub issuer: String,
    pub asset_type: String,
    pub stellar_asset_contract: String,
    pub registered_at: i64,
    pub is_active: bool,
}

/// Mirror of the on-chain Prediction struct from the `prediction-market` contract.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Prediction {
    pub id: i64,
    pub agent: String,
    pub asset_id: String,
    pub prediction_type: String,
    pub value: i64,
    pub confidence: i32,
    pub submitted_at: i64,
    pub resolution_ledger: i64,
    pub status: String,
    pub resolved_value: Option<i64>,
}

/// Mirror of the on-chain ReputationScore struct from the `reputation` contract.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AgentScore {
    pub agent: String,
    pub total_predictions: i32,
    pub correct_predictions: i32,
    pub accuracy_bps: i32,
    pub streak: i32,
    pub last_scored_at: i64,
}

/// Standard JSON error response returned by all endpoints.
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
}
