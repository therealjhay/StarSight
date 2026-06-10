use crate::config::Config;
use crate::models::{AssetMarketData, RawPrediction};

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

/// Errors that can occur during Gemini API analysis.
#[derive(Debug)]
pub enum AnalyzeError {
    /// HTTP request to the Gemini API failed.
    Http(reqwest::Error),
    /// Gemini API returned a non-success status code.
    ApiError(reqwest::StatusCode, String),
    /// Failed to parse the JSON prediction from Gemini's response.
    ParseFailed(String),
}

impl std::fmt::Display for AnalyzeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AnalyzeError::Http(e) => write!(f, "Gemini API request failed: {}", e),
            AnalyzeError::ApiError(code, body) => {
                write!(f, "Gemini API returned status {}: {}", code, body)
            }
            AnalyzeError::ParseFailed(reason) => {
                write!(f, "Failed to parse prediction from Gemini response: {}", reason)
            }
        }
    }
}

impl std::error::Error for AnalyzeError {}

// ---------------------------------------------------------------------------
// Gemini API request/response types
// ---------------------------------------------------------------------------

#[derive(Debug, serde::Serialize)]
struct GeminiRequest {
    #[serde(rename = "systemInstruction")]
    system_instruction: SystemInstruction,
    contents: Vec<GeminiContent>,
}

#[derive(Debug, serde::Serialize)]
struct SystemInstruction {
    parts: Vec<GeminiPart>,
}

#[derive(Debug, serde::Serialize)]
struct GeminiContent {
    parts: Vec<GeminiPart>,
}

#[derive(Debug, serde::Serialize)]
struct GeminiPart {
    text: String,
}

#[derive(Debug, serde::Deserialize)]
struct GeminiResponse {
    candidates: Option<Vec<GeminiCandidate>>,
}

#[derive(Debug, serde::Deserialize)]
struct GeminiCandidate {
    content: Option<GeminiCandidateContent>,
}

#[derive(Debug, serde::Deserialize)]
struct GeminiCandidateContent {
    parts: Option<Vec<GeminiPartResp>>,
}

#[derive(Debug, serde::Deserialize)]
struct GeminiPartResp {
    text: Option<String>,
}

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

const GEMINI_API_BASE: &str = "https://generativelanguage.googleapis.com/v1beta/models/gemini-1.5-flash:generateContent";

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Calls the Gemini API to generate a structured prediction for the given asset.
pub async fn analyze(
    client: &reqwest::Client,
    config: &Config,
    asset: &AssetMarketData,
) -> Result<RawPrediction, AnalyzeError> {
    let system_prompt = build_system_prompt();
    let user_prompt = build_user_prompt(asset);

    let request_body = GeminiRequest {
        system_instruction: SystemInstruction {
            parts: vec![GeminiPart {
                text: system_prompt,
            }],
        },
        contents: vec![GeminiContent {
            parts: vec![GeminiPart { text: user_prompt }],
        }],
    };

    let url = format!("{}?key={}", GEMINI_API_BASE, config.gemini_api_key);

    let response = client
        .post(&url)
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

    let gemini_response: GeminiResponse = response
        .json()
        .await
        .map_err(|e| AnalyzeError::ParseFailed(format!("response deserialization: {}", e)))?;

    // Extract the text content from the first candidate block.
    let text = gemini_response
        .candidates
        .and_then(|c| c.into_iter().next())
        .and_then(|c| c.content)
        .and_then(|c| c.parts)
        .and_then(|p| p.into_iter().next())
        .and_then(|p| p.text)
        .ok_or_else(|| {
            AnalyzeError::ParseFailed("No text content block in Gemini response".to_string())
        })?;

    // Parse the JSON prediction from the text. Gemini may wrap the JSON in
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
