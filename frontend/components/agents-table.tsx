"use client";

import Link from "next/link";
import DataTable, { type Column } from "@/components/data-table";
import AddressCell from "@/components/address-cell";
import type { AgentScore } from "@/lib/types";

interface AgentsTableProps {
  data: AgentScore[];
}

export default function AgentsTable({ data }: AgentsTableProps) {
  const columns: Column<AgentScore>[] = [
    {
      key: "rank",
      header: "#",
      render: (row: AgentScore) => {
        const idx = data.findIndex((a) => a.agent === row.agent);
        return (
          <span className="font-mono text-text-muted text-xs">{idx + 1}</span>
        );
      },
      align: "center",
    },
    {
      key: "agent",
      header: "Agent",
      render: (row) => (
        <Link
          href={`/agents/${encodeURIComponent(row.agent)}`}
          className="hover:text-accent transition-colors duration-normal"
        >
          <AddressCell address={row.agent} />
        </Link>
      ),
      getRowKey: (row) => row.agent,
    },
    {
      key: "accuracy_bps",
      header: "Accuracy",
      render: (row) => (
        <span className="font-mono text-success">
          {(row.accuracy_bps / 100).toFixed(1)}%
        </span>
      ),
      sortValue: (row) => row.accuracy_bps,
      mono: true,
      align: "right",
    },
    {
      key: "streak",
      header: "Streak",
      render: (row) => (
        <span className="font-mono">
          {row.streak > 0 ? (
            <>
              <span aria-hidden="true">🔥</span> {row.streak}
              <span className="sr-only"> streak</span>
            </>
          ) : (
            row.streak
          )}
        </span>
      ),
      sortValue: (row) => row.streak,
      mono: true,
      align: "right",
    },
    {
      key: "total_predictions",
      header: "Total Predictions",
      render: (row) => (
        <span className="font-mono">{row.total_predictions}</span>
      ),
      sortValue: (row) => row.total_predictions,
      mono: true,
      align: "right",
    },
    {
      key: "correct_predictions",
      header: "Correct",
      render: (row) => (
        <span className="font-mono text-text-muted">
          {row.correct_predictions}
        </span>
      ),
      sortValue: (row) => row.correct_predictions,
      mono: true,
      align: "right",
      hideOnMobile: true,
    },
  ];

  return (
    <DataTable
      id="agents-table"
      columns={columns}
      data={data}
      emptyMessage="No agents registered"
    />
  );
}
