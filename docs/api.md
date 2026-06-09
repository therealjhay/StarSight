# StarSight REST API Reference

Base URL: `http://localhost:3001` (configurable via `API_PORT`)

All responses use `Content-Type: application/json`. Error responses follow this shape:

```json
{
  "error": "human-readable message"
}
```

---

## Assets

### List Assets

```
GET /assets
```

Returns all registered RWA assets, newest first.

**Parameters:** None

**Response:** `200 OK`

```json
[
  {
    "id": "US-TBILL-3M",
    "name": "US 3-Month Treasury Bill",
    "issuer": "GBVVY2OH6IADV5F6EV77VR4JTJ7JWNFYFWKZ2KATEDDU2JBZF6NGTAZC",
    "asset_type": "Bond",
    "stellar_asset_contract": "0000000000000000000000000000000000000000000000000000000000000000",
    "registered_at": 1717961234,
    "is_active": true
  }
]
```

---

### Get Asset

```
GET /assets/:id
```

Returns a single asset by its symbol ID.

**Parameters:**

| Name | In   | Type   | Description                |
|------|------|--------|----------------------------|
| `id` | path | string | Asset symbol (e.g. `US-TBILL-3M`) |

**Response:** `200 OK`

```json
{
  "id": "US-TBILL-3M",
  "name": "US 3-Month Treasury Bill",
  "issuer": "GBVVY2OH6IADV5F6EV77VR4JTJ7JWNFYFWKZ2KATEDDU2JBZF6NGTAZC",
  "asset_type": "Bond",
  "stellar_asset_contract": "0000000000000000000000000000000000000000000000000000000000000000",
  "registered_at": 1717961234,
  "is_active": true
}
```

**Error:** `404 Not Found`

```json
{
  "error": "Asset 'UNKNOWN' not found"
}
```

---

## Predictions

### List Predictions

```
GET /predictions
```

Returns all predictions, newest first.

**Parameters:** None

**Response:** `200 OK`

```json
[
  {
    "id": 1,
    "agent": "GBVVY2OH6IADV5F6EV77VR4JTJ7JWNFYFWKZ2KATEDDU2JBZF6NGTAZC",
    "asset_id": "US-TBILL-3M",
    "prediction_type": "YieldForecast",
    "value": 5250000,
    "confidence": 8000,
    "submitted_at": 1717961234,
    "resolution_ledger": 3004899,
    "status": "Pending",
    "resolved_value": null
  }
]
```

---

### Get Prediction

```
GET /predictions/:id
```

Returns a single prediction by its numeric ID.

**Parameters:**

| Name | In   | Type    | Description       |
|------|------|---------|-------------------|
| `id` | path | integer | Prediction ID     |

**Response:** `200 OK`

```json
{
  "id": 1,
  "agent": "GBVVY2OH6IADV5F6EV77VR4JTJ7JWNFYFWKZ2KATEDDU2JBZF6NGTAZC",
  "asset_id": "US-TBILL-3M",
  "prediction_type": "YieldForecast",
  "value": 5250000,
  "confidence": 8000,
  "submitted_at": 1717961234,
  "resolution_ledger": 3004899,
  "status": "Pending",
  "resolved_value": null
}
```

**Error:** `404 Not Found`

```json
{
  "error": "Prediction 999 not found"
}
```

---

### Get Predictions by Agent

```
GET /predictions/agent/:address
```

Returns all predictions submitted by a specific agent.

**Parameters:**

| Name      | In   | Type   | Description                        |
|-----------|------|--------|------------------------------------|
| `address` | path | string | Stellar public key (G... format)   |

**Response:** `200 OK`

```json
[
  {
    "id": 1,
    "agent": "GBVVY2OH6IADV5F6EV77VR4JTJ7JWNFYFWKZ2KATEDDU2JBZF6NGTAZC",
    "asset_id": "US-TBILL-3M",
    "prediction_type": "YieldForecast",
    "value": 5250000,
    "confidence": 8000,
    "submitted_at": 1717961234,
    "resolution_ledger": 3004899,
    "status": "Pending",
    "resolved_value": null
  }
]
```

Returns an empty array `[]` if no predictions exist for the agent.

---

## Agents

### List Agents

```
GET /agents
```

Returns all agents with reputation scores, ranked by accuracy (highest first).

**Parameters:** None

**Response:** `200 OK`

```json
[
  {
    "agent": "GBVVY2OH6IADV5F6EV77VR4JTJ7JWNFYFWKZ2KATEDDU2JBZF6NGTAZC",
    "total_predictions": 10,
    "correct_predictions": 8,
    "accuracy_bps": 8000,
    "streak": 3,
    "last_scored_at": 1717961234
  }
]
```

---

### Get Agent Score

```
GET /agents/:address/score
```

Returns the reputation score for a single agent.

**Parameters:**

| Name      | In   | Type   | Description                        |
|-----------|------|--------|------------------------------------|
| `address` | path | string | Stellar public key (G... format)   |

**Response:** `200 OK`

```json
{
  "agent": "GBVVY2OH6IADV5F6EV77VR4JTJ7JWNFYFWKZ2KATEDDU2JBZF6NGTAZC",
  "total_predictions": 10,
  "correct_predictions": 8,
  "accuracy_bps": 8000,
  "streak": 3,
  "last_scored_at": 1717961234
}
```

**Error:** `404 Not Found`

```json
{
  "error": "Agent 'GXYZ...' not found"
}
```

---

## WebSocket

### Prediction Stream

```
WS /ws
```

Upgrades to a WebSocket connection that receives real-time prediction events. New predictions are broadcast as JSON whenever the indexer detects a `submit_prediction` event on-chain.

**Message Format:** Each message is a JSON-serialized `Prediction` object:

```json
{
  "id": 3,
  "agent": "GBVVY2OH6IADV5F6EV77VR4JTJ7JWNFYFWKZ2KATEDDU2JBZF6NGTAZC",
  "asset_id": "US-TBILL-3M",
  "prediction_type": "PriceTarget",
  "value": 100250000,
  "confidence": 9000,
  "submitted_at": 1717962000,
  "resolution_ledger": 3005399,
  "status": "Pending",
  "resolved_value": null
}
```

**Connection Example (JavaScript):**

```javascript
const ws = new WebSocket("ws://localhost:3001/ws");

ws.onmessage = (event) => {
  const prediction = JSON.parse(event.data);
  console.log("New prediction:", prediction);
};

ws.onclose = () => console.log("Disconnected");
```

---

## Error Codes

| HTTP Code | Meaning                |
|-----------|------------------------|
| `200`     | Success                |
| `404`     | Resource not found     |
| `500`     | Internal server error  |

All error responses use the standard `{ "error": "..." }` shape.

---

## Environment Variables

| Variable                          | Required | Default   | Description                              |
|-----------------------------------|----------|-----------|------------------------------------------|
| `DATABASE_URL`                    | Yes      | —         | PostgreSQL connection string             |
| `API_PORT`                        | No       | `3001`    | HTTP server listen port                  |
| `STELLAR_RPC_URL`                 | Yes      | —         | Soroban RPC endpoint URL                 |
| `STELLAR_NETWORK`                 | No       | `testnet` | Stellar network name                     |
| `ASSET_REGISTRY_CONTRACT_ID`      | Yes      | —         | Deployed asset-registry contract ID      |
| `PREDICTION_MARKET_CONTRACT_ID`   | Yes      | —         | Deployed prediction-market contract ID   |
| `REPUTATION_CONTRACT_ID`          | Yes      | —         | Deployed reputation contract ID          |
| `REWARDS_CONTRACT_ID`             | Yes      | —         | Deployed rewards contract ID             |
