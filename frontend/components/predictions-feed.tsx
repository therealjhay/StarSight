"use client";

import { useState, useEffect, useRef, useCallback } from "react";
import type { Prediction } from "@/lib/types";
import PredictionsTable from "@/components/predictions-table";

const WS_URL = process.env.NEXT_PUBLIC_WS_URL || "ws://localhost:3001/ws";

export default function PredictionsFeed() {
  const [predictions, setPredictions] = useState<Prediction[]>([]);
  const [loading, setLoading] = useState(true);
  const [wsStatus, setWsStatus] = useState<"connecting" | "open" | "closed">("connecting");
  const wsRef = useRef<WebSocket | null>(null);
  const reconnectTimer = useRef<ReturnType<typeof setTimeout> | null>(null);

  // Fetch initial predictions
  useEffect(() => {
    async function fetchInitial() {
      try {
        const res = await fetch("/api/predictions");
        if (res.ok) {
          const data: Prediction[] = await res.json();
          setPredictions(data);
        }
      } catch {
        // API unavailable
      } finally {
        setLoading(false);
      }
    }
    fetchInitial();
  }, []);

  // WebSocket connection
  const connect = useCallback(() => {
    if (wsRef.current?.readyState === WebSocket.OPEN) return;

    setWsStatus("connecting");
    const ws = new WebSocket(WS_URL);
    wsRef.current = ws;

    ws.onopen = () => setWsStatus("open");

    ws.onmessage = (event) => {
      try {
        const prediction: Prediction = JSON.parse(event.data);
        setPredictions((prev) => {
          const next = [prediction, ...prev];
          // Cap at 200 items to prevent memory leak
          return next.slice(0, 200);
        });
      } catch {
        // Invalid message
      }
    };

    ws.onclose = () => {
      setWsStatus("closed");
      // Reconnect after 3 seconds
      reconnectTimer.current = setTimeout(connect, 3000);
    };

    ws.onerror = () => {
      ws.close();
    };
  }, []);

  useEffect(() => {
    connect();
    return () => {
      wsRef.current?.close();
      if (reconnectTimer.current) clearTimeout(reconnectTimer.current);
    };
  }, [connect]);

  return (
    <div>
      {/* Header with WS status */}
      <div className="flex items-center justify-between mb-6">
        <div>
          <h1 className="text-xl font-semibold text-text-primary">Predictions</h1>
          <p className="text-sm text-text-muted mt-1">Live prediction feed</p>
        </div>
        <div
          className="flex items-center gap-2 text-xs text-text-muted"
          role="status"
          aria-live="polite"
          aria-atomic="true"
        >
          <span
            className={`inline-flex relative ${
              wsStatus === "open"
                ? "text-success"
                : wsStatus === "connecting"
                  ? "text-warning"
                  : "text-danger"
            }`}
            aria-hidden="true"
          >
            {wsStatus === "open" && (
              <span className="absolute inset-0 w-2 h-2 rounded-full bg-success animate-pulse-ring" />
            )}
            <span className="relative inline-block w-2 h-2 rounded-full bg-current" />
          </span>
          <span>
            {wsStatus === "open" ? "Live" : wsStatus === "connecting" ? "Connecting..." : "Disconnected"}
          </span>
        </div>
      </div>

      {/* Table */}
      {loading ? (
        <div className="border border-border bg-surface-raised overflow-hidden">
          <table className="w-full text-sm">
            <thead>
              <tr className="border-b border-border bg-surface-raised">
                {[1, 2, 3, 4, 5, 6].map((i) => (
                  <th key={i} className="px-4 py-3"><div className="h-4 bg-surface-overlay rounded animate-pulse" /></th>
                ))}
              </tr>
            </thead>
            <tbody>
              {[1, 2, 3, 4, 5].map((row) => (
                <tr key={row} className="border-b border-border last:border-0">
                  {[1, 2, 3, 4, 5, 6].map((col) => (
                    <td key={col} className="px-4 py-3">
                      <div className="h-4 bg-surface-overlay rounded animate-pulse w-3/4" />
                    </td>
                  ))}
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      ) : predictions.length === 0 ? (
        <div id="predictions-empty" className="border border-border bg-surface-raised p-8 text-center text-text-muted text-sm">
          No data available
        </div>
      ) : (
        <PredictionsTable
          id="predictions-table"
          data={predictions}
          emptyMessage="No predictions available"
          linkAsset
        />
      )}
    </div>
  );
}