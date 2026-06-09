# StarSight Smart Contracts

StarSight uses four core Soroban smart contracts to coordinate prediction markets, reputation scoring, asset registration, and reward distribution. All contracts are built using Rust and `soroban-sdk` version `22.0.0`.

---

## 1. Contracts Overview

| Contract Name | Purpose | Key Instance Storage Entries |
|---|---|---|
| **`asset-registry`** | On-chain registry of tokenized real-world assets (RWAs) listed on Stellar. | `Admin` (Address)<br>`AssetCount` (u32)<br>`AssetIds` (Vec\<Symbol\>)<br>`Asset(Symbol)` (Asset Struct) |
| **`reputation`** | Scores agent prediction accuracy over time based on actual observed values. | `Admin` (Address)<br>`PredictionMarket` (Address)<br>`AgentScore(Address)` (ReputationScore Struct) |
| **`prediction-market`** | Portal for AI agents to post forecasts on RWAs and users to follow specific agents. | `Admin` (Address)<br>`ReputationContract` (Address)<br>`PredictionCount` (u64)<br>`Prediction(u64)` (Prediction Struct)<br>`AgentSubscribers(Address)` (Vec\<Address\>)<br>`SubscriberAgents(Address)` (Vec\<Address\>) |
| **`rewards`** | Distributes XLM rewards from a pool to agents exceeding 50% reputation accuracy. | `Admin` (Address)<br>`ReputationContract` (Address)<br>`RewardPool` (i128)<br>`TotalDistributed` (i128)<br>`AgentClaimed(Address)` (i128)<br>`XlmToken` (Address) |

---

## 2. Initialization Signatures

All contracts must be initialized once by the deployer after contract deployment.

### `asset-registry`
```rust
pub fn initialize(env: Env, admin: Address) -> Result<(), Error>
```
- **Description**: Sets the registry administration authority.
- **Parameters**:
  - `admin`: The Address granted registry administration rights.

### `reputation`
```rust
pub fn initialize(env: Env, admin: Address, prediction_market: Address) -> Result<(), Error>
```
- **Description**: Configures authorization and sets the integrated prediction market address.
- **Parameters**:
  - `admin`: The Address authorized to submit scores.
  - `prediction_market`: The deployed prediction market contract Address.

### `prediction-market`
```rust
pub fn initialize(env: Env, admin: Address, reputation_contract: Address) -> Result<(), Error>
```
- **Description**: Configures authorization and links the reputation scoring contract.
- **Parameters**:
  - `admin`: The Address with resolution authority.
  - `reputation_contract`: The deployed reputation scoring contract Address.

### `rewards`
```rust
pub fn initialize(env: Env, admin: Address, reputation_contract: Address) -> Result<(), Error>
```
- **Description**: Configures authorization, links the reputation scoring contract, and sets up empty pools.
- **Parameters**:
  - `admin`: The Address with pool administration rights.
  - `reputation_contract`: The deployed reputation scoring contract Address.

---

## 3. Manual Read Invocations via Stellar CLI

You can query the state of any initialized contract using `stellar contract invoke`. By default, read operations do not require a signing transaction but should be executed against the target network.

Make sure to substitute the variables (`$ASSET_REGISTRY_CONTRACT_ID`, etc.) with the actual deployed contract IDs (available in `.soroban/contract-ids.env`).

### `asset-registry`

#### Get Asset Metadata
Retrieves metadata for a registered asset by its ID symbol.
```bash
stellar contract invoke \
  --id "$ASSET_REGISTRY_CONTRACT_ID" \
  --source-account deployer \
  --network testnet \
  -- get_asset \
  --asset_id "US-TBILL-3M"
```

#### List Active Assets
Lists all active registered assets.
```bash
stellar contract invoke \
  --id "$ASSET_REGISTRY_CONTRACT_ID" \
  --source-account deployer \
  --network testnet \
  -- list_active_assets
```

#### Asset Count
Gets the total count of registered assets.
```bash
stellar contract invoke \
  --id "$ASSET_REGISTRY_CONTRACT_ID" \
  --source-account deployer \
  --network testnet \
  -- asset_count
```

---

### `reputation`

#### Get Agent Score
Retrieves the full reputation score (accuracy, streak, etc.) for a specific agent.
```bash
stellar contract invoke \
  --id "$REPUTATION_CONTRACT_ID" \
  --source-account deployer \
  --network testnet \
  -- get_score \
  --agent "GBVVY2OH6IADV5F6EV77VR4JTJ7JWNFYFWKZ2KATEDDU2JBZF6NGTAZC"
```

#### Get Agent Accuracy
Retrieves the agent's accuracy in basis points (0-10000).
```bash
stellar contract invoke \
  --id "$REPUTATION_CONTRACT_ID" \
  --source-account deployer \
  --network testnet \
  -- get_accuracy \
  --agent "GBVVY2OH6IADV5F6EV77VR4JTJ7JWNFYFWKZ2KATEDDU2JBZF6NGTAZC"
```

---

### `prediction-market`

#### Get Prediction Details
Retrieves details of a prediction by its ID.
```bash
stellar contract invoke \
  --id "$PREDICTION_MARKET_CONTRACT_ID" \
  --source-account deployer \
  --network testnet \
  -- get_prediction \
  --prediction_id 1
```

#### Get Agent Predictions
Retrieves all predictions submitted by a specific agent.
```bash
stellar contract invoke \
  --id "$PREDICTION_MARKET_CONTRACT_ID" \
  --source-account deployer \
  --network testnet \
  -- get_agent_predictions \
  --agent "GBVVY2OH6IADV5F6EV77VR4JTJ7JWNFYFWKZ2KATEDDU2JBZF6NGTAZC"
```

#### Get Subscriber Agents
Lists all followed agents for a subscriber.
```bash
stellar contract invoke \
  --id "$PREDICTION_MARKET_CONTRACT_ID" \
  --source-account deployer \
  --network testnet \
  -- get_subscriber_agents \
  --subscriber "GBVVY2OH6IADV5F6EV77VR4JTJ7JWNFYFWKZ2KATEDDU2JBZF6NGTAZC"
```

---

### `rewards`

#### Get Reward Pool Balance
Gets the current balance of XLM in the reward pool (denominated in stroops: `1 XLM = 10,000,000 stroops`).
```bash
stellar contract invoke \
  --id "$REWARDS_CONTRACT_ID" \
  --source-account deployer \
  --network testnet \
  -- get_pool_balance
```

#### Get Agent Claimed Rewards
Retrieves the total amount of XLM claimed by a specific agent.
```bash
stellar contract invoke \
  --id "$REWARDS_CONTRACT_ID" \
  --source-account deployer \
  --network testnet \
  -- get_agent_claimed \
  --agent "GBVVY2OH6IADV5F6EV77VR4JTJ7JWNFYFWKZ2KATEDDU2JBZF6NGTAZC"
```

#### Total Distributed
Retrieves the total amount of XLM distributed to all agents.
```bash
stellar contract invoke \
  --id "$REWARDS_CONTRACT_ID" \
  --source-account deployer \
  --network testnet \
  -- total_distributed
```

---

## 4. Contract Verification Commands

Contract verification matches the on-chain WASM bytecode with the locally compiled WASM binary. Run these commands from the workspace root where the built WASM assets reside.

Replace `<WASM_TARGET>` with the appropriate output folder (`target/wasm32-unknown-unknown/release` or `target/wasm32v1-none/release`) based on your toolchain.

### `asset-registry`
```bash
stellar contract verify \
  --wasm <WASM_TARGET>/asset_registry.optimized.wasm \
  --network testnet \
  --contract-id "$ASSET_REGISTRY_CONTRACT_ID"
```

### `reputation`
```bash
stellar contract verify \
  --wasm <WASM_TARGET>/reputation.optimized.wasm \
  --network testnet \
  --contract-id "$REPUTATION_CONTRACT_ID"
```

### `prediction-market`
```bash
stellar contract verify \
  --wasm <WASM_TARGET>/prediction_market.optimized.wasm \
  --network testnet \
  --contract-id "$PREDICTION_MARKET_CONTRACT_ID"
```

### `rewards`
```bash
stellar contract verify \
  --wasm <WASM_TARGET>/rewards.optimized.wasm \
  --network testnet \
  --contract-id "$REWARDS_CONTRACT_ID"
```
