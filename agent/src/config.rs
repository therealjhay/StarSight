/// Typed configuration for the StarSight agent, loaded from environment variables.
#[derive(Clone, Debug)]
pub struct Config {
    /// Base URL of the StarSight backend API (e.g. `http://localhost:3001`).
    pub api_base_url: String,
    /// Anthropic Claude API key for generating predictions.
    pub claude_api_key: String,
    /// Stellar Soroban RPC endpoint URL.
    pub stellar_rpc_url: String,
    /// Stellar network passphrase (e.g. "Test SDF Network ; September 2015").
    pub network_passphrase: String,
    /// Secret key for the agent's Stellar keypair.
    // TODO: In production, this should be a dedicated agent keypair, not the
    // deployer key. Using DEPLOYER_SECRET_KEY for now as a development shortcut.
    pub deployer_secret_key: String,
    /// Contract ID of the deployed prediction-market Soroban contract.
    pub prediction_market_contract_id: String,
    /// Seconds between each poll cycle.
    pub poll_interval_secs: u64,
}

impl Config {
    /// Reads configuration from environment variables.
    ///
    /// # Panics
    /// Panics if any required variable is missing or unparseable.
    pub fn from_env() -> Self {
        let api_port = std::env::var("API_PORT").unwrap_or_else(|_| "3001".into());
        Self {
            api_base_url: format!("http://localhost:{}", api_port),
            claude_api_key: required_env("CLAUDE_API_KEY"),
            stellar_rpc_url: required_env("STELLAR_RPC_URL"),
            network_passphrase: required_env("STELLAR_NETWORK_PASSPHRASE"),
            deployer_secret_key: required_env("DEPLOYER_SECRET_KEY"),
            prediction_market_contract_id: required_env("PREDICTION_MARKET_CONTRACT_ID"),
            poll_interval_secs: std::env::var("AGENT_POLL_INTERVAL_SECS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(60),
        }
    }
}

/// Reads a required environment variable or panics with a descriptive message.
fn required_env(key: &str) -> String {
    std::env::var(key)
        .unwrap_or_else(|_| panic!("Required environment variable {} is not set", key))
}
