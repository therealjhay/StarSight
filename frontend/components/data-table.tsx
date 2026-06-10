"use client";

import { useState, useMemo, type ReactNode } from "react";

export interface Column<T> {
  key: string;
  header: string;
  render: (row: T) => ReactNode;
  sortValue?: (row: T) => number | string;
  mono?: boolean;
  align?: "left" | "right" | "center";
}

interface DataTableProps<T> {
  columns: Column<T>[];
  data: T[];
  emptyMessage?: string;
  id?: string;
}

export default function DataTable<T>({
  columns,
  data,
  emptyMessage = "No data available",
  id = "data-table",
}: DataTableProps<T>) {
  const [sortKey, setSortKey] = useState<string | null>(null);
  const [sortDir, setSortDir] = useState<"asc" | "desc">("desc");

  const handleSort = (key: string) => {
    if (sortKey === key) {
      setSortDir((d) => (d === "asc" ? "desc" : "asc"));
    } else {
      setSortKey(key);
      setSortDir("desc");
    }
  };

  const sorted = useMemo(() => {
    if (!sortKey) return data;

    const col = columns.find((c) => c.key === sortKey);
    if (!col?.sortValue) return data;

    const fn = col.sortValue;
    return [...data].sort((a, b) => {
      const av = fn(a);
      const bv = fn(b);

      if (typeof av === "number" && typeof bv === "number") {
        return sortDir === "asc" ? av - bv : bv - av;
      }

      const sa = String(av);
      const sb = String(bv);
      return sortDir === "asc" ? sa.localeCompare(sb) : sb.localeCompare(sa);
    });
  }, [data, sortKey, sortDir, columns]);

  if (data.length === 0) {
    return (
      <div
        id={`${id}-empty`}
        className="border border-border bg-surface-raised p-8 text-center text-text-muted text-sm"
      >
        {emptyMessage}
      </div>
    );
  }

  return (
    <div className="overflow-x-auto border border-border animate-fade-in">
      <table id={id} className="w-full text-sm">
        <thead>
          <tr className="border-b border-border bg-surface-raised">
            {columns.map((col) => {
              const isSorted = sortKey === col.key;
              const canSort = !!col.sortValue;
              return (
                <th
                  key={col.key}
                  className={`px-4 py-3 text-xs uppercase tracking-widest text-text-muted font-medium whitespace-nowrap ${
                    col.align === "right"
                      ? "text-right"
                      : col.align === "center"
                        ? "text-center"
                        : "text-left"
                  } ${canSort ? "cursor-pointer select-none hover:text-text-primary" : ""}`}
                  onClick={canSort ? () => handleSort(col.key) : undefined}
                >
                  {col.header}
                  {isSorted && (
                    <span className="ml-1 text-accent">
                      {sortDir === "asc" ? "↑" : "↓"}
                    </span>
                  )}
                </th>
              );
            })}
          </tr>
        </thead>
        <tbody>
          {sorted.map((row, i) => (
            <tr
              key={i}
              className="border-b border-border last:border-b-0 hover:bg-surface-overlay transition-colors duration-100"
            >
              {columns.map((col) => (
                <td
                  key={col.key}
                  className={`px-4 py-3 whitespace-nowrap ${
                    col.mono ? "font-mono" : ""
                  } ${
                    col.align === "right"
                      ? "text-right"
                      : col.align === "center"
                        ? "text-center"
                        : "text-left"
                  }`}
                >
                  {col.render(row)}
                </td>
              ))}
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}
