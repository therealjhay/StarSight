"use client";

import DataTable, { type Column } from "@/components/data-table";
import AddressCell from "@/components/address-cell";
import PredictionBadge from "@/components/prediction-badge";
import type { Prediction } from "@/lib/types";
import { formatDate } from "@/lib/format";

interface PredictionsTableProps {
  data: Prediction[];
  emptyMessage?: string;
  id?: string;
  linkAsset?: boolean;
}

export default function PredictionsTable({
  data,
  emptyMessage = "No predictions available",
  id = "predictions-table",
  linkAsset = true,
}: PredictionsTableProps) {
  const columns: Column<Prediction>[] = [
    {
      key: "id",
      header: "ID",
      mono: true,
      render: (r) => (
        <span className="font-mono text-text-muted">#{r.id}</span>
      ),
      sortValue: (r) => r.id,
      getRowKey: (r) => r.id,
    },
    {
      key: "agent",
      header: "Agent",
      render: (r) => <AddressCell address={r.agent} />,
    },
    {
      key: "asset_id",
      header: "Asset",
      mono: true,
      render: (r) =>
        linkAsset ? (
          <a
            href={`/assets/${encodeURIComponent(r.asset_id)}`}
            className="text-accent hover:underline font-mono text-sm"
          >
            {r.asset_id}
          </a>
        ) : (
          <span className="font-mono text-sm">{r.asset_id}</span>
        ),
      sortValue: (r) => r.asset_id,
    },
    {
      key: "prediction_type",
      header: "Type",
      render: (r) => (
        <span className="text-xs uppercase tracking-wider text-text-muted">
          {r.prediction_type}
        </span>
      ),
      sortValue: (r) => r.prediction_type,
      hideOnMobile: true,
    },
    {
      key: "value",
      header: "Value",
      mono: true,
      align: "right",
      render: (r) => (
        <span className="font-mono">{r.value.toLocaleString()}</span>
      ),
      sortValue: (r) => r.value,
    },
    {
      key: "confidence",
      header: "Confidence",
      mono: true,
      align: "right",
      render: (r) => (
        <span className="font-mono">
          {(r.confidence / 100).toFixed(1)}%
        </span>
      ),
      sortValue: (r) => r.confidence,
      hideOnMobile: true,
    },
    {
      key: "status",
      header: "Status",
      render: (r) => <PredictionBadge status={r.status} />,
      sortValue: (r) => r.status,
    },
    {
      key: "submitted_at",
      header: "Submitted",
      mono: true,
      render: (r) => (
        <span className="font-mono text-text-muted text-xs">
          {formatDate(r.submitted_at)}
        </span>
      ),
      sortValue: (r) => r.submitted_at,
      hideOnMobile: true,
    },
  ];

  return (
    <DataTable id={id} columns={columns} data={data} emptyMessage={emptyMessage} />
  );
}
