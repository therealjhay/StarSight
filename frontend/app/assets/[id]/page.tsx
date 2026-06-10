import type { Metadata } from "next";
import { notFound } from "next/navigation";
import { getAsset, getPredictionsByAsset } from "@/lib/api";
import AddressCell from "@/components/address-cell";
import PredictionsTable from "@/components/predictions-table";
import type { Prediction } from "@/lib/types";

interface PageProps {
  params: { id: string };
}

export async function generateMetadata({ params }: PageProps): Promise<Metadata> {
  return {
    title: `${params.id} — StarSight`,
    description: `Asset details and prediction history for ${params.id}.`,
  };
}

export default async function AssetDetailPage({ params }: PageProps) {
  let asset;
  let predictions: Prediction[] = [];

  try {
    asset = await getAsset(params.id);
  } catch {
    notFound();
  }

  try {
    predictions = await getPredictionsByAsset(params.id);
  } catch {
    // Predictions unavailable
  }

  return (
    <div>
      <div className="mb-6">
        <h1 className="text-xl font-semibold text-text-primary font-mono">
          {asset.id}
        </h1>
        <p className="text-sm text-text-muted mt-1">{asset.name}</p>
      </div>

      {/* Metadata */}
      <div className="border border-border bg-surface-raised p-5 mb-6 animate-fade-in">
        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4 text-sm">
          <div>
            <p className="text-xs uppercase tracking-widest text-text-muted mb-1">Type</p>
            <p className="text-text-primary">{asset.asset_type}</p>
          </div>
          <div>
            <p className="text-xs uppercase tracking-widest text-text-muted mb-1">Issuer</p>
            <AddressCell address={asset.issuer} />
          </div>
          <div>
            <p className="text-xs uppercase tracking-widest text-text-muted mb-1">Contract</p>
            <AddressCell address={asset.stellar_asset_contract} />
          </div>
          <div>
            <p className="text-xs uppercase tracking-widest text-text-muted mb-1">Status</p>
            <span className={`inline-flex items-center gap-1.5 text-sm ${asset.is_active ? "text-success" : "text-danger"}`}>
              <span className={`inline-block w-2 h-2 rounded-full ${asset.is_active ? "bg-success" : "bg-danger"}`} />
              {asset.is_active ? "Active" : "Inactive"}
            </span>
          </div>
        </div>
      </div>

      {/* Prediction History */}
      <div className="mb-4">
        <h2 className="text-lg font-semibold text-text-primary">Prediction History</h2>
        <p className="text-sm text-text-muted mt-1">
          {predictions.length} prediction{predictions.length !== 1 ? "s" : ""}
        </p>
      </div>

      <PredictionsTable
        id="asset-predictions-table"
        data={predictions}
        emptyMessage="No predictions for this asset"
      />
    </div>
  );
}
