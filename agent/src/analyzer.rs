use crate::config::Config;
use crate::models::{AssetMarketData, RawPrediction};

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

/// Errors that can occur during Claude API analysis.
#[derive(Debug)]
pub enum AnalyzeError {
    /// HTTP request to the Claude API failed.
    Http(reqwest::Error),
    /// Claude API returned a non-success status code.
    ApiError(reqwest::StatusCode, String),
    /// Failed to parse the JSON prediction from Claude's response.
    ParseFailed(String),
}

impl std::fmt::Display for AnalyzeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AnalyzeError::Http(e) => write!(f, "Claude API request failed: {}", e),
            AnalyzeError::ApiError(code, body) => {
                write!(f, "Claude API returned status {}: {}", code, body)
            }
            AnalyzeError::ParseFailed(reason) => {
                write!(f, "Failed to parse prediction from Claude response: {}", reason)
            }
        }
    }
}

impl std::error::Error for AnalyzeError {}

// ---------------------------------------------------------------------------
// Claude API request/response types
// ---------------------------------------------------------------------------

/// Request body for the Anthropic Messages API.
#[derive(Debug, serde::Serialize)]
struct ClaudeRequest {
    model: String,
    max_tokens: u32,
    system: String,
    messages: Vec<ClaudeMessage>,
}

/// A single message in the Claude conversation.
#[derive(Debug, serde::Serialize)]
struct ClaudeMessage {
    role: String,
    content: String,
}

/// Top-level response from the Claude Messages API.
#[derive(Debug, serde::Deserialize)]
struct ClaudeResponse {
    content: Vec<ContentBlock>,
}

/// A content block in the Claude response.
#[derive(Debug, serde::Deserialize)]
struct ContentBlock {
    #[serde(rename = "type")]
    block_type: String,
    text: Option<String>,
}

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

const CLAUDE_API_URL: &str = "https://api.anthropic.com/v1/messages";
const CLAUDE_MODEL: &str = "claude-sonnet-4-20250514";
const ANTHROPIC_VERSION: &str = "2023-06-01";

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Calls the Claude API to generate a structured prediction for the given asset.
///
/// The system prompt frames Claude as a financial analyst. The user prompt
/// contains the asset's market data. The response is parsed strictly; if
/// parsing fails, an [`AnalyzeError::ParseFailed`] is returned (the caller
/// should log and skip the asset).
pub async fn analyze(
    client: &reqwest::Client,
    config: &Config,
    asset: &AssetMarketData,
) -> Result<RawPrediction, AnalyzeError> {
    let system_prompt = build_system_prompt();
    let user_prompt = build_user_prompt(asset);

    let request_body = ClaudeRequest {
        model: CLAUDE_MODEL.to_string(),
        max_tokens: 1024,
        system: system_prompt,
        messages: vec![ClaudeMessage {
            role: "user".to_string(),
            content: user_prompt,
        }],
    };

    let response = client
        .post(CLAUDE_API_URL)
        .header("x-api-key", &config.claude_api_key)
        .header("anthropic-version", ANTHROPIC_VERSION)
        .header("content-type", "application/json")
        .json(&request_body)
        .send()
        .await
        .map_err(AnalyzeError::Http)?;

    let status = response.status();
    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        return Err(AnalyzeError::ApiError(status, body));
    }

    let claude_response: ClaudeResponse = response
        .json()
        .await
        .map_err(|e| AnalyzeError::ParseFailed(format!("response deserialization: {}", e)))?;

    // Extract the text content from the first text block.
    let text = claude_response
        .content
        .iter()
        .find_map(|block| {
            if block.block_type == "text" {
                block.text.clone()
            } else {
                None
            }
        })
        .ok_or_else(|| {
            AnalyzeError::ParseFailed("No text content block in Claude response".to_string())
        })?;

    // Parse the JSON prediction from the text. Claude may wrap the JSON in
    // markdown code fences, so we strip those if present.
    let json_str = extract_json(&text);

    let prediction: RawPrediction = serde_json::from_str(json_str).map_err(|e| {
        AnalyzeError::ParseFailed(format!(
            "JSON parse error: {}. Raw text: {}",
            e,
            &text[..text.len().min(500)]
        ))
    })?;

    // Validate confidence range.
    if prediction.confidence > 10_000 {
        return Err(AnalyzeError::ParseFailed(format!(
            "confidence {} exceeds max 10000",
            prediction.confidence
        )));
    }

    Ok(prediction)
}

// ---------------------------------------------------------------------------
// Prompt construction
// ---------------------------------------------------------------------------

fn build_system_prompt() -> String {
    r#"You are a senior quantitative financial analyst specializing in tokenized real-world assets (RWAs) on blockchain networks. You have deep expertise in REITs, treasury bonds, commodity-backed tokens, and other asset-backed securities.

Your task is to analyze the provided asset market data and generate a structured prediction. You must respond with ONLY a single JSON object (no markdown, no explanation outside the JSON) matching this exact schema:

{
  "prediction_type": "PriceTarget" | "YieldForecast" | "RiskScore",
  "value": <integer — predicted value scaled by 1_000_000>,
  "confidence": <integer from 0 to 10000 — basis points representing your confidence>,
  "reasoning": "<one sentence explaining your prediction>"
}

Rules:
- Choose the most appropriate prediction_type based on the asset data.
- For PriceTarget: value is the predicted price scaled by 1_000_000.
- For YieldForecast: value is the predicted annualized yield in basis points, scaled by 1_000_000.
- For RiskScore: value is a risk score from 0 (no risk) to 100, scaled by 1_000_000.
- confidence must be between 0 and 10000 (basis points).
- If you cannot make a confident prediction, return confidence: 0.
- Do NOT wrap the JSON in markdown code fences. Return raw JSON only."#.to_string()
}

fn build_user_prompt(asset: &AssetMarketData) -> String {
    format!(
        r#"Analyze the following tokenized RWA asset and provide your prediction:

Asset ID: {}
Name: {}
Type: {}
Current Price: {} (scaled by 1_000_000)
24h Price Change: {:.2}%
Current Yield: {} bps
30-Day Average Yield: {} bps
30-Day Volatility: {:.2}%
Market Cap: {} (scaled by 1_000_000)
Last Updated: {}"#,
        asset.asset_id,
        asset.name,
        asset.asset_type,
        asset.current_price,
        asset.price_change_24h,
        asset.current_yield,
        asset.avg_yield_30d,
        asset.volatility_30d,
        asset.market_cap,
        asset.last_updated,
    )
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Extracts JSON from text that may be wrapped in markdown code fences.
fn extract_json(text: &str) -> &str {
    let trimmed = text.trim();

    // Try to strip ```json ... ``` or ``` ... ```
    if let Some(rest) = trimmed.strip_prefix("```json") {
        if let Some(json) = rest.strip_suffix("```") {
            return json.trim();
        }
    }
    if let Some(rest) = trimmed.strip_prefix("```") {
        if let Some(json) = rest.strip_suffix("```") {
            return json.trim();
        }
    }

    // Look for the first { and last } to extract embedded JSON.
    if let (Some(start), Some(end)) = (trimmed.find('{'), trimmed.rfind('}')) {
        if start < end {
            return &trimmed[start..=end];
        }
    }

    trimmed
}
