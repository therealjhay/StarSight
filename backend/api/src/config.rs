/// Typed configuration loaded from environment variables.
#[derive(Clone, Debug)]
pub struct Config {
    /// PostgreSQL connection string (e.g. `postgres://user:pass@host/db`).
    pub database_url: String,
    /// Port the HTTP server binds to.
    pub api_port: u16,
    /// Stellar Soroban RPC endpoint URL.
    pub stellar_rpc_url: String,
    /// Stellar network name (testnet, mainnet, etc.).
    pub stellar_network: String,

    // Contract IDs to index events from.
    pub asset_registry_contract_id: String,
    pub prediction_market_contract_id: String,
    pub reputation_contract_id: String,
    pub rewards_contract_id: String,
}

impl Config {
    /// Reads configuration from environment variables.
    ///
    /// # Panics
    /// Panics if any required variable is missing.
    pub fn from_env() -> Self {
        Self {
            database_url: required_env("DATABASE_URL"),
            api_port: std::env::var("API_PORT")
                .unwrap_or_else(|_| "3001".into())
                .parse()
                .expect("API_PORT must be a valid u16"),
            stellar_rpc_url: required_env("STELLAR_RPC_URL"),
            stellar_network: std::env::var("STELLAR_NETWORK")
                .unwrap_or_else(|_| "testnet".into()),
            asset_registry_contract_id: required_env("ASSET_REGISTRY_CONTRACT_ID"),
            prediction_market_contract_id: required_env("PREDICTION_MARKET_CONTRACT_ID"),
            reputation_contract_id: required_env("REPUTATION_CONTRACT_ID"),
            rewards_contract_id: required_env("REWARDS_CONTRACT_ID"),
        }
    }
}

fn required_env(key: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| panic!("Required environment variable {} is not set", key))
}
