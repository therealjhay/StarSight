#!/usr/bin/env bash

set -euo pipefail

# Determine script and root directories
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

echo "====================================================================="
echo " StarSight Test Data Seeding"
echo "====================================================================="

# 1. Source and validate .env file
ENV_FILE="$ROOT_DIR/.env"
if [ -f "$ENV_FILE" ]; then
  echo "Sourcing environment from $ENV_FILE..."
  set -a
  source "$ENV_FILE"
  set +a
else
  echo "Error: .env file not found at $ENV_FILE" >&2
  exit 1
fi

# Ensure mandatory variables are set
if [ -z "${STELLAR_NETWORK:-}" ]; then
  echo "Error: STELLAR_NETWORK is not set in .env" >&2
  exit 1
fi
if [ -z "${DEPLOYER_SECRET_KEY:-}" ]; then
  echo "Error: DEPLOYER_SECRET_KEY is not set in .env" >&2
  exit 1
fi

# 2. Source contract IDs
IDS_FILE="$ROOT_DIR/.soroban/contract-ids.env"
if [ -f "$IDS_FILE" ]; then
  echo "Sourcing contract IDs from $IDS_FILE..."
  set -a
  source "$IDS_FILE"
  set +a
else
  echo "Error: Contract IDs file not found at $IDS_FILE. Please run deploy.sh first." >&2
  exit 1
fi

# Validate contract IDs are present
if [ -z "${ASSET_REGISTRY_CONTRACT_ID:-}" ] || \
   [ -z "${PREDICTION_MARKET_CONTRACT_ID:-}" ] || \
   [ -z "${REPUTATION_CONTRACT_ID:-}" ] || \
   [ -z "${REWARDS_CONTRACT_ID:-}" ]; then
  echo "Error: One or more contract IDs are missing in $IDS_FILE" >&2
  exit 1
fi

# 3. Register deployer identity in stellar CLI temporarily
echo "Configuring deployer identity..."
echo "$DEPLOYER_SECRET_KEY" | stellar keys add deployer --secret-key --overwrite > /dev/null
DEPLOYER_ADDRESS=$(stellar keys address deployer)
echo "Deployer Address: $DEPLOYER_ADDRESS"

# 4. Fetch Native XLM Token Address dynamically
echo "Fetching Native XLM Token Address..."
XLM_TOKEN_ADDRESS=$(stellar contract id asset --asset native --network "$STELLAR_NETWORK")
echo "Native XLM Token Address: $XLM_TOKEN_ADDRESS"

# 5. Seed RWA Assets (Idempotent)
echo "-----------------------------------------------------"
echo " Seeding Assets..."
echo "-----------------------------------------------------"

# Helper to register an asset if it is not already registered
register_asset_if_missing() {
  local asset_id="$1"
  local name="$2"
  local asset_type="$3"

  echo "Checking if asset '$asset_id' is registered..."
  if stellar contract invoke \
      --id "$ASSET_REGISTRY_CONTRACT_ID" \
      --source deployer \
      --network "$STELLAR_NETWORK" \
      -- get_asset \
      --asset_id "$asset_id" >/dev/null 2>&1; then
    echo "Asset '$asset_id' already exists. Skipping registration."
  else
    echo "Registering asset '$asset_id' ($name)..."
    stellar contract invoke \
      --id "$ASSET_REGISTRY_CONTRACT_ID" \
      --source deployer \
      --network "$STELLAR_NETWORK" \
      -- register_asset \
      --asset "{\"id\": \"$asset_id\", \"name\": \"$name\", \"issuer\": \"$DEPLOYER_ADDRESS\", \"asset_type\": \"$asset_type\", \"stellar_asset_contract\": \"0000000000000000000000000000000000000000000000000000000000000000\", \"registered_at\": $(date +%s), \"is_active\": true}"
    echo "Confirmed: Asset '$asset_id' registered successfully."
  fi
}

register_asset_if_missing "US-TBILL-3M" "US 3-Month Treasury Bill" "Bond"
register_asset_if_missing "NYC-REIT-A" "NYC Real Estate Investment Trust" "RealEstate"
register_asset_if_missing "GOLD-SPOT" "Gold Spot Commodity" "Commodity"

# 6. Seed Predictions (Idempotent)
echo "-----------------------------------------------------"
echo " Seeding Predictions..."
echo "-----------------------------------------------------"

# Determine Horizon URL to get current ledger sequence
HORIZON_URL="https://horizon-testnet.stellar.org"
if [ "$STELLAR_NETWORK" = "public" ] || [ "$STELLAR_NETWORK" = "mainnet" ]; then
  HORIZON_URL="https://horizon.stellar.org"
elif [ "$STELLAR_NETWORK" = "local" ] || [ "$STELLAR_NETWORK" = "standalone" ]; then
  HORIZON_URL="http://localhost:8000"
fi

echo "Fetching latest ledger sequence from $HORIZON_URL..."
CURRENT_LEDGER=$(curl -s "$HORIZON_URL" | jq '.history_latest_ledger' 2>/dev/null || true)
if ! [[ "$CURRENT_LEDGER" =~ ^[0-9]+$ ]]; then
  echo "Warning: Could not fetch latest ledger from Horizon. Defaulting to a safe sequence number."
  CURRENT_LEDGER=3000000
fi
echo "Latest ledger sequence: $CURRENT_LEDGER"

# Check if predictions have already been submitted by the deployer agent
echo "Checking existing predictions for agent $DEPLOYER_ADDRESS..."
PREDICTIONS_JSON=$(stellar contract invoke \
  --id "$PREDICTION_MARKET_CONTRACT_ID" \
  --source deployer \
  --network "$STELLAR_NETWORK" \
  -- get_agent_predictions \
  --agent "$DEPLOYER_ADDRESS" 2>/dev/null || echo "[]")

# Extract prediction count
PRED_COUNT=$(echo "$PREDICTIONS_JSON" | jq '. | length' 2>/dev/null || echo "0")
echo "Found $PRED_COUNT predictions for agent $DEPLOYER_ADDRESS."

if [ "$PRED_COUNT" -ge 2 ]; then
  echo "Agent already has $PRED_COUNT predictions. Skipping prediction seeding."
else
  # Submit Prediction 1
  RESOLUTION_LEDGER_1=$((CURRENT_LEDGER + 1000))
  echo "Submitting YieldForecast prediction for US-TBILL-3M (Resolution Ledger: $RESOLUTION_LEDGER_1)..."
  pred_id_1=$(stellar contract invoke \
    --id "$PREDICTION_MARKET_CONTRACT_ID" \
    --source deployer \
    --network "$STELLAR_NETWORK" \
    -- submit_prediction \
    --agent "$DEPLOYER_ADDRESS" \
    --asset_id "US-TBILL-3M" \
    --prediction_type "YieldForecast" \
    --value 5250000 \
    --confidence 8000 \
    --resolution_ledger "$RESOLUTION_LEDGER_1")
  echo "Confirmed: Submitted prediction 1 (ID: $pred_id_1)"

  # Submit Prediction 2
  RESOLUTION_LEDGER_2=$((CURRENT_LEDGER + 1500))
  echo "Submitting PriceTarget prediction for US-TBILL-3M (Resolution Ledger: $RESOLUTION_LEDGER_2)..."
  pred_id_2=$(stellar contract invoke \
    --id "$PREDICTION_MARKET_CONTRACT_ID" \
    --source deployer \
    --network "$STELLAR_NETWORK" \
    -- submit_prediction \
    --agent "$DEPLOYER_ADDRESS" \
    --asset_id "US-TBILL-3M" \
    --prediction_type "PriceTarget" \
    --value 100250000 \
    --confidence 9000 \
    --resolution_ledger "$RESOLUTION_LEDGER_2")
  echo "Confirmed: Submitted prediction 2 (ID: $pred_id_2)"
fi

echo "====================================================================="
echo " Seeding completed successfully."
echo "====================================================================="
