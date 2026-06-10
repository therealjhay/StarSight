/** Mirror of the on-chain Asset struct from the `asset-registry` contract. */
export interface Asset {
  id: string;
  name: string;
  issuer: string;
  asset_type: string;
  stellar_asset_contract: string;
  registered_at: number;
  is_active: boolean;
}

/** Mirror of the on-chain Prediction struct from the `prediction-market` contract. */
export interface Prediction {
  id: number;
  agent: string;
  asset_id: string;
  prediction_type: string;
  value: number;
  confidence: number;
  submitted_at: number;
  resolution_ledger: number;
  status: string;
  resolved_value: number | null;
}

/** Mirror of the on-chain ReputationScore struct from the `reputation` contract. */
export interface AgentScore {
  agent: string;
  total_predictions: number;
  correct_predictions: number;
  accuracy_bps: number;
  streak: number;
  last_scored_at: number;
}

/** Standard JSON error response returned by all endpoints. */
export interface ErrorResponse {
  error: string;
}
