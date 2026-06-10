use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Data fetched from the StarSight backend API (GET /assets/market-data)
// ---------------------------------------------------------------------------

/// Pre-aggregated market data for a single RWA asset, returned by the backend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetMarketData {
    /// On-chain asset identifier (e.g. "REIT01").
    pub asset_id: String,
    /// Human-readable name.
    pub name: String,
    /// Asset category (e.g. "RealEstate", "Treasury", "Commodity").
    pub asset_type: String,
    /// Current price scaled by 1_000_000.
    pub current_price: i64,
    /// 24-hour price change as a percentage (e.g. -2.3).
    pub price_change_24h: f64,
    /// Current annualized yield in basis points (e.g. 540 = 5.40%).
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

// ---------------------------------------------------------------------------
// Claude API prediction response
// ---------------------------------------------------------------------------

/// The kind of prediction the agent can submit.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PredictionType {
    PriceTarget,
    YieldForecast,
    RiskScore,
}

impl PredictionType {
    /// Returns the string representation expected by the Soroban contract enum.
    pub fn as_symbol_str(&self) -> &'static str {
        match self {
            PredictionType::PriceTarget => "PriceTarget",
            PredictionType::YieldForecast => "YieldForecast",
            PredictionType::RiskScore => "RiskScore",
        }
    }
}

impl std::fmt::Display for PredictionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_symbol_str())
    }
}

/// Raw prediction parsed from Claude's JSON response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawPrediction {
    /// One of: "PriceTarget", "YieldForecast", "RiskScore".
    pub prediction_type: PredictionType,
    /// Predicted value scaled by 1_000_000.
    pub value: i64,
    /// Confidence score from 0 to 10_000 basis points.
    pub confidence: u32,
    /// One-sentence reasoning from the model.
    pub reasoning: String,
}

// ---------------------------------------------------------------------------
// Submitted prediction tracking
// ---------------------------------------------------------------------------

/// Record of a prediction that was successfully submitted on-chain.
#[derive(Debug, Clone)]
pub struct SubmittedPrediction {
    /// On-chain prediction ID returned by the contract.
    pub prediction_id: u64,
    /// Asset the prediction was made for.
    pub asset_id: String,
    /// Type of prediction submitted.
    pub prediction_type: PredictionType,
    /// The predicted value.
    pub value: i64,
    /// Confidence score.
    pub confidence: u32,
    /// UTC timestamp when the submission was confirmed.
    pub submitted_at: chrono::DateTime<chrono::Utc>,
}
