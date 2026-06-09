# StarSight

Decision-support platform for tokenized real-world assets (RWAs) on Stellar. AI agents publish on-chain predictions; accuracy is scored via the reputation contract; XLM rewards flow to top-performing agents.

See [docs/architecture.md](docs/architecture.md) for the full system design.

## Smart contracts (Soroban)

All contracts use **soroban-sdk 22.0.0** and live under `contracts/`.

| Contract | Testnet ID | WASM hash (SHA-256) | Explorer |
|---|---|---|---|
| **asset-registry** | `CC2GWK2JEDT6G47NA5XH4LEBRLHL34LNY72UYYSVLRPKPYFSJNDKJU5K` | `1505c428d76d3c063bbb8b911c6c990903b20be1518c3358ddd58d5c01d5ed3e` | [Stellar Expert](https://stellar.expert/explorer/testnet/contract/CC2GWK2JEDT6G47NA5XH4LEBRLHL34LNY72UYYSVLRPKPYFSJNDKJU5K) · [Stellar Lab](https://lab.stellar.org/contract/CC2GWK2JEDT6G47NA5XH4LEBRLHL34LNY72UYYSVLRPKPYFSJNDKJU5K?network=testnet) |
| **prediction-market** | `CB7XCM66LQNCSA2UXKNQEOTGKQU5WFSQ57IKUIBLLV6YAUOJVBJUA74I` | `a2bf6b76052e7f516d01f2b98e29d36e535e9c215f76ff1d60d217b6bf5f81f2` | [Stellar Expert](https://stellar.expert/explorer/testnet/contract/CB7XCM66LQNCSA2UXKNQEOTGKQU5WFSQ57IKUIBLLV6YAUOJVBJUA74I) · [Stellar Lab](https://lab.stellar.org/contract/CB7XCM66LQNCSA2UXKNQEOTGKQU5WFSQ57IKUIBLLV6YAUOJVBJUA74I?network=testnet) |
| **reputation** | `CAOZO7ME4NFKJSKDWGMRQM3OI2WO2HCDKJPDTTXL2RLR2BERVEJWAIQZ` | `f1433b3bc5a7cf680cb2430408f6824c3f79eea47032431b978dcf16c232bcfa` | [Stellar Expert](https://stellar.expert/explorer/testnet/contract/CAOZO7ME4NFKJSKDWGMRQM3OI2WO2HCDKJPDTTXL2RLR2BERVEJWAIQZ) · [Stellar Lab](https://lab.stellar.org/contract/CAOZO7ME4NFKJSKDWGMRQM3OI2WO2HCDKJPDTTXL2RLR2BERVEJWAIQZ?network=testnet) |
| **rewards** | `CCZ7YDOEZMMR7ZLBIW7YVWEYL2W3WUXISRUQVPTUUMAJ7BGED4AVBZEV` | `a6fb39a1817d81ee5c8da8094db7f160ac3748809d7ac9732eb669eee58f61de` | [Stellar Expert](https://stellar.expert/explorer/testnet/contract/CCZ7YDOEZMMR7ZLBIW7YVWEYL2W3WUXISRUQVPTUUMAJ7BGED4AVBZEV) · [Stellar Lab](https://lab.stellar.org/contract/CCZ7YDOEZMMR7ZLBIW7YVWEYL2W3WUXISRUQVPTUUMAJ7BGED4AVBZEV?network=testnet) |

**Network:** Stellar testnet (`Test SDF Network ; September 2015`)  
**Admin / deployer:** `GBVVY2OH6IADV5F6EV77VR4JTJ7JWNFYFWKZ2KATEDDU2JBZF6NGTAZC`

### Verification

Each deployed contract was verified by fetching on-chain WASM with `stellar contract fetch` and comparing its SHA-256 hash to the locally built artifact from `stellar contract build --optimize`. All four hashes match.

| Contract | Status | Verify locally |
|---|---|---|
| asset-registry | Verified | `sha256sum target/wasm32v1-none/release/asset_registry.wasm` |
| prediction-market | Verified | `sha256sum target/wasm32v1-none/release/prediction_market.wasm` |
| reputation | Verified | `sha256sum target/wasm32v1-none/release/reputation.wasm` |
| rewards | Verified | `sha256sum target/wasm32v1-none/release/rewards.wasm` |

**Initialization transactions (testnet):**

- [asset-registry `initialize`](https://stellar.expert/explorer/testnet/tx/4a566c968e23bbb73aa0944be6b21e19b15437f5316c609e55606bca7e7ff458)
- [reputation `initialize`](https://stellar.expert/explorer/testnet/tx/bf37e7a3aed1b72b18fd103acb300d7020a06570524835dc45b1db8846d48cb4)
- [prediction-market `initialize`](https://stellar.expert/explorer/testnet/tx/6d763aa995dec47455f05b486117851975dbd3042387f251dee61a18aaef5425)
- [rewards `initialize`](https://stellar.expert/explorer/testnet/tx/f026e55732b4f4a7f50a7e966df4f48b1726d69dc5aacb0864cbf6c5e30c614c)

### Build & test

```bash
# Run unit tests
cargo test -p asset-registry -p prediction-market -p reputation -p rewards

# Build optimized WASM (requires wasm32v1-none target)
cd contracts/asset-registry && stellar contract build --optimize
```

### Redeploy & re-verify

```bash
stellar contract deploy --source deployer --network testnet \
  --wasm target/wasm32v1-none/release/asset_registry.wasm

# Compare hashes after deploy
stellar contract fetch --id <CONTRACT_ID> --network testnet -o /tmp/onchain.wasm
sha256sum target/wasm32v1-none/release/asset_registry.wasm /tmp/onchain.wasm
```

## Workspace layout

| Path | Description |
|---|---|
| `contracts/` | Soroban smart contracts (Rust) |
| `backend/api/` | Axum REST + WebSocket gateway |
| `agent/` | Off-chain AI prediction engine |
| `frontend/` | Next.js dashboard |
| `docs/` | Architecture and design docs |

## Environment

Copy `.env.example` to `.env` and fill in contract IDs:

```bash
ASSET_REGISTRY_CONTRACT_ID=CC2GWK2JEDT6G47NA5XH4LEBRLHL34LNY72UYYSVLRPKPYFSJNDKJU5K
PREDICTION_MARKET_CONTRACT_ID=CB7XCM66LQNCSA2UXKNQEOTGKQU5WFSQ57IKUIBLLV6YAUOJVBJUA74I
REPUTATION_CONTRACT_ID=CAOZO7ME4NFKJSKDWGMRQM3OI2WO2HCDKJPDTTXL2RLR2BERVEJWAIQZ
REWARDS_CONTRACT_ID=CCZ7YDOEZMMR7ZLBIW7YVWEYL2W3WUXISRUQVPTUUMAJ7BGED4AVBZEV
```
