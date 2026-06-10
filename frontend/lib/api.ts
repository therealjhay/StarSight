import type { Asset, Prediction, AgentScore } from "./types";

const BASE =
  typeof window === "undefined"
    ? process.env.NEXT_PUBLIC_API_URL || "http://localhost:3001"
    : "/api";

async function get<T>(path: string): Promise<T> {
  const res = await fetch(`${BASE}${path}`, {
    cache: "no-store",
  });

  if (!res.ok) {
    throw new Error(`API ${res.status}: ${res.statusText}`);
  }

  return res.json() as Promise<T>;
}

// ─── Assets ──────────────────────────────────────────────────────────────────

export async function getAssets(): Promise<Asset[]> {
  return get<Asset[]>("/assets");
}

export async function getAsset(id: string): Promise<Asset> {
  return get<Asset>(`/assets/${encodeURIComponent(id)}`);
}

// ─── Predictions ─────────────────────────────────────────────────────────────

export async function getPredictions(): Promise<Prediction[]> {
  return get<Prediction[]>("/predictions");
}

export async function getPrediction(id: number): Promise<Prediction> {
  return get<Prediction>(`/predictions/${id}`);
}

export async function getPredictionsByAgent(
  address: string
): Promise<Prediction[]> {
  return get<Prediction[]>(`/predictions/agent/${encodeURIComponent(address)}`);
}

// ─── Agents ──────────────────────────────────────────────────────────────────

export async function getAgents(): Promise<AgentScore[]> {
  return get<AgentScore[]>("/agents");
}

export async function getAgentScore(address: string): Promise<AgentScore> {
  return get<AgentScore>(
    `/agents/${encodeURIComponent(address)}/score`
  );
}

// ─── Predictions by Asset ────────────────────────────────────────────────────

export async function getPredictionsByAsset(
  assetId: string
): Promise<Prediction[]> {
  const all = await getPredictions();
  return all.filter((p) => p.asset_id === assetId);
}
