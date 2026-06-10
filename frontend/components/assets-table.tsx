"use client";

import Link from "next/link";
import DataTable, { type Column } from "@/components/data-table";
import AddressCell from "@/components/address-cell";
import type { Asset } from "@/lib/types";

const columns: Column<Asset>[] = [
  {
    key: "id",
    header: "Symbol",
    render: (row) => (
      <Link
        href={`/assets/${encodeURIComponent(row.id)}`}
        className="text-accent hover:underline font-mono text-sm"
      >
        {row.id}
      </Link>
    ),
    sortValue: (row) => row.id,
    mono: true,
  },
  {
    key: "name",
    header: "Name",
    render: (row) => <span className="text-text-primary">{row.name}</span>,
    sortValue: (row) => row.name,
  },
  {
    key: "asset_type",
    header: "Type",
    render: (row) => (
      <span className="text-text-muted text-xs uppercase tracking-wider">
        {row.asset_type}
      </span>
    ),
    sortValue: (row) => row.asset_type,
  },
  {
    key: "issuer",
    header: "Issuer",
    render: (row) => <AddressCell address={row.issuer} />,
  },
  {
    key: "is_active",
    header: "Status",
    render: (row) => (
      <span
        className={`inline-block w-2 h-2 rounded-full ${
          row.is_active ? "bg-success" : "bg-danger"
        }`}
        title={row.is_active ? "Active" : "Inactive"}
      />
    ),
    sortValue: (row) => (row.is_active ? 1 : 0),
    align: "center",
  },
];

interface AssetsTableProps {
  data: Asset[];
}

export default function AssetsTable({ data }: AssetsTableProps) {
  return (
    <DataTable
      id="assets-table"
      columns={columns}
      data={data}
      emptyMessage="No assets registered"
    />
  );
}
