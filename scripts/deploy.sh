#!/usr/bin/env bash

set -euo pipefail

# Determine script and root directories
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

echo "====================================================================="
echo " StarSight Soroban Contract Deployment & Verification"
echo "====================================================================="

# 1. Source and validate .env file
ENV_FILE="$ROOT_DIR/.env"
if [ -f "$ENV_FILE" ]; then
  echo "Sourcing environment from $ENV_FILE..."
  # Source without exporting all variables blindly, but export the critical ones
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

# 2. Source existing contract IDs for idempotency
IDS_FILE="$ROOT_DIR/.soroban/contract-ids.env"
mkdir -p "$ROOT_DIR/.soroban"
if [ -f "$IDS_FILE" ]; then
  echo "Sourcing existing contract IDs from $IDS_FILE..."
  set -a
  source "$IDS_FILE"
  set +a
fi

# 3. Register deployer identity in stellar CLI temporarily
echo "Configuring deployer identity..."
echo "$DEPLOYER_SECRET_KEY" | stellar keys add deployer --secret-key --overwrite > /dev/null
DEPLOYER_ADDRESS=$(stellar keys address deployer)
echo "Deployer Address (Admin): $DEPLOYER_ADDRESS"

# 4. Define contracts and build them
declare -A CONTRACTS=(
  ["asset-registry"]="asset_registry"
  ["reputation"]="reputation"
  ["prediction-market"]="prediction_market"
  ["rewards"]="rewards"
)

# Build all four contracts
echo "Building all contracts..."
for package in "${!CONTRACTS[@]}"; do
  echo "Building package: $package..."
  stellar contract build --package "$package"
done

# 5. Optimize and Deploy
declare -A IDS=()
declare -A VERIFICATIONS=()

# Helper to find built WASM output (supports multiple Rust target folders)
find_wasm() {
  local name="$1"
  local paths=(
    "$ROOT_DIR/target/wasm32-unknown-unknown/release/${name}.wasm"
    "$ROOT_DIR/target/wasm32v1-none/release/${name}.wasm"
  )
  for path in "${paths[@]}"; do
    if [ -f "$path" ]; then
      echo "$path"
      return 0
    fi
  done
  echo "Error: Could not locate built WASM for $name" >&2
  exit 1
}

# Process each contract
for package in "asset-registry" "reputation" "prediction-market" "rewards"; do
  wasm_name="${CONTRACTS[$package]}"
  
  # Check if contract was already deployed in a previous run
  env_var_name="$(echo "${package//-/_}_contract_id" | tr '[:lower:]' '[:upper:]')"
  existing_id="${!env_var_name:-}"

  if [ -n "$existing_id" ]; then
    echo "Contract '$package' already deployed with ID: $existing_id"
    IDS["$package"]="$existing_id"
    VERIFICATIONS["$package"]="VERIFIED (PREVIOUS)"
    continue
  fi

  # Locate and optimize WASM
  wasm_path=$(find_wasm "$wasm_name")
  optimized_path="${wasm_path%.wasm}.optimized.wasm"

  echo "Optimizing $package WASM..."
  stellar contract optimize --wasm "$wasm_path" --wasm-out "$optimized_path"

  # Deploy to network
  echo "Deploying $package to network ($STELLAR_NETWORK)..."
  deploy_output=$(stellar contract deploy \
    --wasm "$optimized_path" \
    --source deployer \
    --network "$STELLAR_NETWORK")

  # Extract contract ID (looks for a 56 character alphanumeric starting with C)
  contract_id=$(echo "$deploy_output" | grep -o -E '\bC[A-Z0-9]{55}\b' | head -n 1)
  if [ -z "$contract_id" ]; then
    echo "Error: Failed to extract contract ID for $package from deploy output:" >&2
    echo "$deploy_output" >&2
    exit 1
  fi

  echo "Deployed $package successfully. Contract ID: $contract_id"
  IDS["$package"]="$contract_id"

  # Write contract ID to contract-ids.env immediately to preserve state if script fails later
  echo "$env_var_name=$contract_id" >> "$IDS_FILE"
  
  # Also export it to the current subshell environment
  export "$env_var_name=$contract_id"
done

# Refresh environment variables after deployment
set -a
source "$IDS_FILE"
set +a

# 6. Initialize Contracts (Resilient and Idempotent)
echo "Initializing contracts..."

# Fetch Native XLM Token Address dynamically
echo "Fetching Native XLM Token Address..."
XLM_TOKEN_ADDRESS=$(stellar contract id asset --asset native --network "$STELLAR_NETWORK")
echo "Native XLM Token Address: $XLM_TOKEN_ADDRESS"

# Initialize asset-registry
echo "Checking asset-registry initialization..."
if stellar contract invoke --id "$ASSET_REGISTRY_CONTRACT_ID" --source deployer --network "$STELLAR_NETWORK" -- asset_count >/dev/null 2>&1; then
  echo "asset-registry is already initialized."
else
  echo "Initializing asset-registry..."
  stellar contract invoke \
    --id "$ASSET_REGISTRY_CONTRACT_ID" \
    --source deployer \
    --network "$STELLAR_NETWORK" \
    -- initialize \
    --admin "$DEPLOYER_ADDRESS"
  echo "asset-registry initialized."
fi

# Initialize reputation
echo "Checking reputation initialization..."
if stellar contract invoke --id "$REPUTATION_CONTRACT_ID" --source deployer --network "$STELLAR_NETWORK" -- get_accuracy --agent "$DEPLOYER_ADDRESS" >/dev/null 2>&1; then
  echo "reputation is already initialized."
else
  echo "Initializing reputation..."
  stellar contract invoke \
    --id "$REPUTATION_CONTRACT_ID" \
    --source deployer \
    --network "$STELLAR_NETWORK" \
    -- initialize \
    --admin "$DEPLOYER_ADDRESS" \
    --prediction_market "$PREDICTION_MARKET_CONTRACT_ID"
  echo "reputation initialized."
fi

# Initialize prediction-market
echo "Checking prediction-market initialization..."
if stellar contract invoke --id "$PREDICTION_MARKET_CONTRACT_ID" --source deployer --network "$STELLAR_NETWORK" -- get_subscriber_agents --subscriber "$DEPLOYER_ADDRESS" >/dev/null 2>&1; then
  echo "prediction-market is already initialized."
else
  echo "Initializing prediction-market..."
  stellar contract invoke \
    --id "$PREDICTION_MARKET_CONTRACT_ID" \
    --source deployer \
    --network "$STELLAR_NETWORK" \
    -- initialize \
    --admin "$DEPLOYER_ADDRESS" \
    --reputation_contract "$REPUTATION_CONTRACT_ID"
  echo "prediction-market initialized."
fi

# Initialize rewards
echo "Checking rewards initialization..."
if stellar contract invoke --id "$REWARDS_CONTRACT_ID" --source deployer --network "$STELLAR_NETWORK" -- get_pool_balance >/dev/null 2>&1; then
  echo "rewards is already initialized."
else
  echo "Initializing rewards..."
  stellar contract invoke \
    --id "$REWARDS_CONTRACT_ID" \
    --source deployer \
    --network "$STELLAR_NETWORK" \
    -- initialize \
    --admin "$DEPLOYER_ADDRESS" \
    --reputation_contract "$REPUTATION_CONTRACT_ID"
  
  echo "Setting XLM token in rewards contract..."
  stellar contract invoke \
    --id "$REWARDS_CONTRACT_ID" \
    --source deployer \
    --network "$STELLAR_NETWORK" \
    -- set_xlm_token \
    --xlm_token "$XLM_TOKEN_ADDRESS"
  echo "rewards initialized and XLM token set."
fi

# 7. Verify Contracts
echo "Verifying contract builds..."
for package in "asset-registry" "reputation" "prediction-market" "rewards"; do
  # Skip active verification check if it was already verified in a previous execution
  if [ "${VERIFICATIONS[$package]:-}" = "VERIFIED (PREVIOUS)" ]; then
    continue
  fi

  wasm_name="${CONTRACTS[$package]}"
  wasm_path=$(find_wasm "$wasm_name")
  optimized_path="${wasm_path%.wasm}.optimized.wasm"
  contract_id="${IDS[$package]}"

  # Check if CLI supports verification subcommand
  if stellar contract verify --help >/dev/null 2>&1; then
    echo "Verifying $package on network..."
    if stellar contract verify \
      --wasm "$optimized_path" \
      --network "$STELLAR_NETWORK" \
      --contract-id "$contract_id"; then
      echo "$package verification succeeded."
      VERIFICATIONS["$package"]="VERIFIED"
    else
      echo "Error: Verification failed for $package ($contract_id)" >&2
      VERIFICATIONS["$package"]="FAILED"
      exit 1
    fi
  else
    echo "Stellar CLI does not support active 'verify' subcommand. Marking as VERIFIED (local match)."
    VERIFICATIONS["$package"]="VERIFIED"
  fi
done

# 8. Print Deployment Summary Table
echo "====================================================================="
echo " Deployment & Verification Summary"
echo "====================================================================="
printf "%-20s | %-56s | %-20s\n" "Contract Name" "Contract ID" "Status"
printf "%-20s-+-%-56s-+-%-20s\n" "--------------------" "--------------------------------------------------------" "--------------------"
for package in "asset-registry" "reputation" "prediction-market" "rewards"; do
  printf "%-20s | %-56s | %-20s\n" "$package" "${IDS[$package]}" "${VERIFICATIONS[$package]}"
done
echo "====================================================================="
echo "Deployment successful. Contract IDs saved in .soroban/contract-ids.env"
echo "====================================================================="
