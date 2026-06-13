"use client";

import { useState, useMemo, type ReactNode } from "react";

export interface Column<T> {
  key: string;
  header: string;
  render: (row: T) => ReactNode;
  sortValue?: (row: T) => number | string;
  mono?: boolean;
  align?: "left" | "right" | "center";
  getRowKey?: (row: T) => string | number;
  hideOnMobile?: boolean;
}

interface DataTableProps<T> {
  columns: Column<T>[];
  data: T[];
  emptyMessage?: string;
  id?: string;
}

function TableView<T>({
  columns,
  data,
  id,
  sortKey,
  sortDir,
  handleSort,
  getRowKey,
}: {
  columns: Column<T>[];
  data: T[];
  id: string;
  sortKey: string | null;
  sortDir: "asc" | "desc";
  handleSort: (key: string) => void;
  getRowKey?: (row: T) => string | number;
}) {
  return (
    <div className="overflow-x-auto border border-border animate-fade-in hidden lg:block">
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
                  }`}
                  scope="col"
                  aria-sort={canSort ? (isSorted ? (sortDir === "asc" ? "ascending" : "descending") : "none") : undefined}
                >
                  {canSort ? (
                    <button
                      type="button"
                      onClick={() => handleSort(col.key)}
                      className="flex items-center gap-1 w-full py-2 hover:text-text-primary focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-accent focus-visible:ring-offset-2 focus-visible:ring-offset-surface-base rounded min-h-[44px]"
                      aria-pressed={isSorted}
                    >
                      {col.header}
                      {isSorted && (
                        <span className="ml-1 text-accent" aria-hidden="true">
                          {sortDir === "asc" ? "↑" : "↓"}
                        </span>
                      )}
                    </button>
                  ) : (
                    <span className="block py-2">{col.header}</span>
                  )}
                </th>
              );
            })}
          </tr>
        </thead>
        <tbody>
          {data.map((row, i) => (
            <tr
              key={getRowKey ? getRowKey(row) : i}
              className="border-b border-border last:border-b-0 hover:bg-surface-overlay transition-colors duration-100"
            >
              {columns.map((col) => (
                <td
                  key={col.key}
                  className={`px-4 py-3 whitespace-nowrap min-h-[44px] ${
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

function CardView<T>({
  columns,
  data,
  id,
  getRowKey,
}: {
  columns: Column<T>[];
  data: T[];
  id: string;
  getRowKey?: (row: T) => string | number;
}) {
  const visibleColumns = columns.filter((c) => !c.hideOnMobile);

  return (
    <div className="block lg:hidden animate-fade-in" role="list" aria-label={id}>
      {data.map((row, i) => (
        <article
          key={getRowKey ? getRowKey(row) : i}
          className="border border-border bg-surface-raised p-4 mb-3 min-h-[44px]"
          role="listitem"
        >
          <dl className="grid gap-3 sm:grid-cols-2">
            {visibleColumns.map((col) => (
              <div key={col.key} className="flex flex-col gap-1">
                <dt className="text-xs uppercase tracking-widest text-text-muted font-medium">
                  {col.header}
                </dt>
                <dd className={`text-sm ${col.mono ? "font-mono" : ""}`}>
                  {col.render(row)}
                </dd>
              </div>
            ))}
          </dl>
        </article>
      ))}
    </div>
  );
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

  const getRowKey = columns.find((c) => c.getRowKey)?.getRowKey;

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
    <>
      <CardView
        id={id}
        columns={columns}
        data={sorted}
        getRowKey={getRowKey}
      />
      <TableView
        id={id}
        columns={columns}
        data={sorted}
        sortKey={sortKey}
        sortDir={sortDir}
        handleSort={handleSort}
        getRowKey={getRowKey}
      />
    </>
  );
}