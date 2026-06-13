import StatCard from "@/components/stat-card";
import { getAssets, getAgents, getPredictions } from "@/lib/api";

export default async function DashboardPage() {
  let totalAssets = 0;
  let predictionsToday = 0;
  let topAgentAccuracy = "—";

  const [assets, agents, predictions] = await Promise.all([
    getAssets(),
    getAgents(),
    getPredictions(),
  ]);

  totalAssets = assets.length;

  // Count predictions submitted in the last 24 hours
  const oneDayAgo = Math.floor(Date.now() / 1000) - 86400;
  predictionsToday = predictions.filter(
    (p) => p.submitted_at > oneDayAgo
  ).length;

  // Top agent by accuracy
  if (agents.length > 0) {
    const top = agents.reduce((best, a) =>
      a.accuracy_bps > best.accuracy_bps ? a : best
    );
    topAgentAccuracy = `${(top.accuracy_bps / 100).toFixed(1)}%`;
  }

  return (
    <div>
      <div className="mb-6">
        <h1 className="text-xl font-semibold text-text-primary">Dashboard</h1>
        <p className="text-sm text-text-muted mt-1">
          StarSight network overview
        </p>
      </div>

      <div className="grid grid-cols-1 sm:grid-cols-3 gap-4">
        <StatCard label="Total Assets" value={totalAssets} sub="Registered RWAs" delay={0} />
        <StatCard
          label="Predictions Today"
          value={predictionsToday}
          sub="Last 24 hours"
          delay={100}
        />
        <StatCard
          label="Top Agent Accuracy"
          value={topAgentAccuracy}
          sub="Highest accuracy score"
          delay={200}
        />
      </div>
    </div>
  );
}
