# StarSight

StarSight is an advanced decentralized Real-World Asset (RWA) prediction market built on the Stellar network using Soroban smart contracts. It seamlessly integrates a high-performance Next.js frontend with an AI-driven Rust daemon to automatically assess asset performance and predict market movements on-chain. This platform allows human users to participate in predictions alongside specialized autonomous agents.

## Architecture Diagram

```text
+-------------------+       +-------------------+       +-------------------+
|                   |       |                   |       |                   |
|  Frontend (Next)  |<----->| Backend API (Rust)|<----->| PostgreSQL (DB)   |
|                   |       |                   |       |                   |
+-------------------+       +---------+---------+       +-------------------+
                                      | ^
                                      | |
                                      v |
                            +---------+---------+
                            |                   |
                            | Soroban Contracts |
                            |                   |
                            +---------+---------+
                                      | ^
                                      | |
                                      v |
                            +---------+---------+
                            |                   |
                            |  AI Agent (Rust)  |
                            |                   |
                            +-------------------+
```

## Prerequisites
- Rust (stable)
- Node.js (20+)
- Stellar CLI (v22+)
- Docker & Docker Compose
- PostgreSQL 16

## Local Setup

1. **Clone the Repository**:
   ```bash
   git clone <repository_url>
   cd StarSight
   ```
2. **Configure Environment Variables**:
   ```bash
   cp .env.example .env
   ```
   Open `.env` and fill in the required values (like your `GEMINI_API_KEY` and `DEPLOYER_SECRET_KEY`).
3. **Start the Database**:
   ```bash
   docker compose up -d postgres
   ```
4. **Build the Backend and Agent**:
   ```bash
   cargo build
   ```
5. **Install Frontend Dependencies**:
   ```bash
   cd frontend && npm install
   ```

## Deploy Contracts

To deploy the Soroban contracts to the Stellar Testnet, use the deployment script:
```bash
./scripts/deploy.sh
```
- **Compiles & Optimizes**: Automatically builds all four contracts (`asset-registry`, `prediction-market`, `reputation`, and `rewards`) for the `wasm32-unknown-unknown` target.
- **Deploys**: Deploys them sequentially to the network using the `DEPLOYER_SECRET_KEY`.
- **Verifies**: Verifies the compiled WebAssembly on-chain to ensure transparency.

### Deployed Contracts (Testnet)

| Contract | Address |
|---|---|
| **Asset Registry** | `CC2GWK2JEDT6G47NA5XH4LEBRLHL34LNY72UYYSVLRPKPYFSJNDKJU5K` |
| **Prediction Market**| `CB7XCM66LQNCSA2UXKNQEOTGKQU5WFSQ57IKUIBLLV6YAUOJVBJUA74I` |
| **Reputation** | `CAOZO7ME4NFKJSKDWGMRQM3OI2WO2HCDKJPDTTXL2RLR2BERVEJWAIQZ` |
| **Rewards** | `CCZ7YDOEZMMR7ZLBIW7YVWEYL2W3WUXISRUQVPTUUMAJ7BGED4AVBZEV` |

## Run Backend

To run the event indexer and REST/WebSocket API:
```bash
cargo run -p starsight-api
```

## Run Agent

To run the AI prediction daemon:
```bash
cargo run -p starsight-agent
```

## Run Frontend

To start the Next.js frontend in development mode:
```bash
cd frontend && npm run dev
```

## Environment Variables

| Key | Required | Description |
|---|---|---|
| `STELLAR_NETWORK` | Yes | The target Stellar network (e.g., `testnet`). |
| `STELLAR_RPC_URL` | Yes | URL for the Soroban RPC endpoint. |
| `STELLAR_NETWORK_PASSPHRASE` | Yes | Network passphrase (e.g., `"Test SDF Network ; September 2015"`). |
| `DEPLOYER_SECRET_KEY` | Yes | Secret key of the account deploying contracts and running the agent. |
| `ASSET_REGISTRY_CONTRACT_ID` | Yes | On-chain ID of the asset registry contract. |
| `PREDICTION_MARKET_CONTRACT_ID`| Yes | On-chain ID of the prediction market contract. |
| `REPUTATION_CONTRACT_ID` | Yes | On-chain ID of the reputation contract. |
| `REWARDS_CONTRACT_ID` | Yes | On-chain ID of the rewards contract. |
| `GEMINI_API_KEY` | Yes | Google Gemini API key used by the AI agent to analyze predictions. |
| `DATABASE_URL` | Yes | PostgreSQL connection string. |
| `API_PORT` | No | Port for the backend API (defaults to 3001). |
| `AGENT_POLL_INTERVAL_SECS` | No | Polling frequency for the AI agent (defaults to 60). |

## Final Audit Checklist

| Check | Status | Notes |
|---|---|---|
| All contracts compile (`cargo check --target wasm32-unknown-unknown`) | PASS | Passed successfully. Run with explicit package targets to avoid compiling host libs. |
| All contract tests pass (`cargo test`) | PASS | Passed successfully. |
| Backend compiles (`cargo build -p api`) | PASS | Passed successfully (`starsight-api`). |
| Agent compiles (`cargo build -p agent`) | PASS | Passed successfully (`starsight-agent`). |
| No circular workspace dependencies | PASS | Verified in workspace `Cargo.toml`. |
| No contract imports another contract's lib crate | PASS | Contracts are logically separated. |
| All `.env.example` keys have corresponding config.rs reads | PASS | Validated against both `backend/api/src/config.rs` and `agent/src/config.rs`. |
| Frontend builds (`npm run build`) | PASS | Passed successfully. Next.js statically optimized pages. |
| `deploy.sh` is executable and idempotent | PASS | Validated. State tracked via local cache and contract query checks. |
| All four contracts verified on testnet in `deploy.sh` | PASS | Explicitly handled in section 7 of the script. |


| **asset-registry** | `CC2GWK2JEDT6G47NA5XH4LEBRLHL34LNY72UYYSVLRPKPYFSJNDKJU5K` | `1505c428d76d3c063bbb8b911c6c990903b20be1518c3358ddd58d5c01d5ed3e` | [Stellar Expert](https://stellar.expert/explorer/testnet/contract/CC2GWK2JEDT6G47NA5XH4LEBRLHL34LNY72UYYSVLRPKPYFSJNDKJU5K) · [Stellar Lab](https://lab.stellar.org/contract/CC2GWK2JEDT6G47NA5XH4LEBRLHL34LNY72UYYSVLRPKPYFSJNDKJU5K?network=testnet) |
| **prediction-market** | `CB7XCM66LQNCSA2UXKNQEOTGKQU5WFSQ57IKUIBLLV6YAUOJVBJUA74I` | `a2bf6b76052e7f516d01f2b98e29d36e535e9c215f76ff1d60d217b6bf5f81f2` | [Stellar Expert](https://stellar.expert/explorer/testnet/contract/CB7XCM66LQNCSA2UXKNQEOTGKQU5WFSQ57IKUIBLLV6YAUOJVBJUA74I) · [Stellar Lab](https://lab.stellar.org/contract/CB7XCM66LQNCSA2UXKNQEOTGKQU5WFSQ57IKUIBLLV6YAUOJVBJUA74I?network=testnet) |
| **reputation** | `CAOZO7ME4NFKJSKDWGMRQM3OI2WO2HCDKJPDTTXL2RLR2BERVEJWAIQZ` | `f1433b3bc5a7cf680cb2430408f6824c3f79eea47032431b978dcf16c232bcfa` | [Stellar Expert](https://stellar.expert/explorer/testnet/contract/CAOZO7ME4NFKJSKDWGMRQM3OI2WO2HCDKJPDTTXL2RLR2BERVEJWAIQZ) · [Stellar Lab](https://lab.stellar.org/contract/CAOZO7ME4NFKJSKDWGMRQM3OI2WO2HCDKJPDTTXL2RLR2BERVEJWAIQZ?network=testnet) |
| **rewards** | `CCZ7YDOEZMMR7ZLBIW7YVWEYL2W3WUXISRUQVPTUUMAJ7BGED4AVBZEV` | `a6fb39a1817d81ee5c8da8094db7f160ac3748809d7ac9732eb669eee58f61de` | [Stellar Expert](https://stellar.expert/explorer/testnet/contract/CCZ7YDOEZMMR7ZLBIW7YVWEYL2W3WUXISRUQVPTUUMAJ7BGED4AVBZEV) · [Stellar Lab](https://lab.stellar.org/contract/CCZ7YDOEZMMR7ZLBIW7YVWEYL2W3WUXISRUQVPTUUMAJ7BGED4AVBZEV?network=testnet) |

**Network:** Stellar testnet (`Test SDF Network ; September 2015`)  
**Admin / deployer:** `GBVVY2OH6IADV5F6EV77VR4JTJ7JWNFYFWKZ2KATEDDU2JBZF6NGTAZC`

## License

This project is licensed under the MIT License.
