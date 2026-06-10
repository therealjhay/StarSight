"use client";

import { useState, useEffect, useRef, useCallback } from "react";
import type { Prediction } from "@/lib/types";
import PredictionBadge from "@/components/prediction-badge";
import AddressCell from "@/components/address-cell";

const WS_URL = "ws://localhost:3001/ws";

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
        setPredictions((prev) => [prediction, ...prev]);
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
        <div className="flex items-center gap-2 text-xs text-text-muted">
          <span
            className={`inline-block w-2 h-2 rounded-full ${
              wsStatus === "open"
                ? "bg-success"
                : wsStatus === "connecting"
                  ? "bg-warning"
                  : "bg-danger"
            }`}
          />
          {wsStatus === "open" ? "Live" : wsStatus === "connecting" ? "Connecting..." : "Disconnected"}
        </div>
      </div>

      {/* Table */}
      {loading ? (
        <div className="border border-border bg-surface-raised p-8 text-center text-text-muted text-sm">
          Loading...
        </div>
      ) : predictions.length === 0 ? (
        <div id="predictions-empty" className="border border-border bg-surface-raised p-8 text-center text-text-muted text-sm">
          No data available
        </div>
      ) : (
        <div className="overflow-x-auto border border-border animate-fade-in">
          <table id="predictions-table" className="w-full text-sm">
            <thead>
              <tr className="border-b border-border bg-surface-raised">
                <th className="px-4 py-3 text-xs uppercase tracking-widest text-text-muted font-medium text-left">ID</th>
                <th className="px-4 py-3 text-xs uppercase tracking-widest text-text-muted font-medium text-left">Agent</th>
                <th className="px-4 py-3 text-xs uppercase tracking-widest text-text-muted font-medium text-left">Asset</th>
                <th className="px-4 py-3 text-xs uppercase tracking-widest text-text-muted font-medium text-left">Type</th>
                <th className="px-4 py-3 text-xs uppercase tracking-widest text-text-muted font-medium text-right">Value</th>
                <th className="px-4 py-3 text-xs uppercase tracking-widest text-text-muted font-medium text-right">Confidence</th>
                <th className="px-4 py-3 text-xs uppercase tracking-widest text-text-muted font-medium text-left">Status</th>
                <th className="px-4 py-3 text-xs uppercase tracking-widest text-text-muted font-medium text-left">Submitted</th>
              </tr>
            </thead>
            <tbody>
              {predictions.map((p) => (
                <tr
                  key={`${p.id}-${p.submitted_at}`}
                  className="border-b border-border last:border-b-0 hover:bg-surface-overlay transition-colors duration-100 animate-fade-in"
                >
                  <td className="px-4 py-3 font-mono text-text-muted whitespace-nowrap">#{p.id}</td>
                  <td className="px-4 py-3 whitespace-nowrap"><AddressCell address={p.agent} /></td>
                  <td className="px-4 py-3 whitespace-nowrap">
                    <a href={`/assets/${encodeURIComponent(p.asset_id)}`} className="text-accent hover:underline font-mono text-sm">{p.asset_id}</a>
                  </td>
                  <td className="px-4 py-3 whitespace-nowrap text-xs uppercase tracking-wider text-text-muted">{p.prediction_type}</td>
                  <td className="px-4 py-3 whitespace-nowrap font-mono text-right">{p.value.toLocaleString()}</td>
                  <td className="px-4 py-3 whitespace-nowrap font-mono text-right">{(p.confidence / 100).toFixed(1)}%</td>
                  <td className="px-4 py-3 whitespace-nowrap"><PredictionBadge status={p.status} /></td>
                  <td className="px-4 py-3 whitespace-nowrap font-mono text-text-muted text-xs">
                    {new Date(p.submitted_at * 1000).toISOString().slice(0, 19).replace("T", " ")}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </div>
  );
}
