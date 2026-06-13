import type { Metadata } from "next";
import { notFound } from "next/navigation";
import { getAgentScore, getPredictionsByAgent } from "@/lib/api";
import StatCard from "@/components/stat-card";
import AddressCell from "@/components/address-cell";
import PredictionsTable from "@/components/predictions-table";
import type { Prediction } from "@/lib/types";

interface PageProps {
  params: { address: string };
}

export async function generateMetadata({ params }: PageProps): Promise<Metadata> {
  const short = params.address.length > 10
    ? `${params.address.slice(0, 6)}...${params.address.slice(-4)}`
    : params.address;
  return {
    title: `Agent ${short} — StarSight`,
    description: `Reputation and prediction history for agent ${short}.`,
  };
}

export default async function AgentDetailPage({ params }: PageProps) {
  let score;
  try {
    score = await getAgentScore(params.address);
  } catch {
    notFound();
  }

  const predictions: Prediction[] = await getPredictionsByAgent(params.address);

  return (
    <div>
      <div className="mb-6">
        <h1 className="text-xl font-semibold text-text-primary">Agent Detail</h1>
        <div className="mt-1">
          <AddressCell address={score.agent} />
        </div>
      </div>

      {/* Score Card */}
      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4 mb-8">
        <StatCard
          label="Accuracy"
          value={`${(score.accuracy_bps / 100).toFixed(1)}%`}
          sub={`${score.correct_predictions} / ${score.total_predictions} correct`}
          delay={0}
        />
        <StatCard label="Total Predictions" value={score.total_predictions} delay={100} />
        <StatCard
          label="Streak"
          value={score.streak > 0 ? `🔥 ${score.streak}` : `${score.streak}`}
          delay={200}
        />
        <StatCard
          label="Last Scored"
          value={
            score.last_scored_at
              ? new Date(score.last_scored_at * 1000).toISOString().slice(0, 10)
              : "—"
          }
          delay={300}
        />
      </div>

      {/* Prediction History */}
      <div className="mb-4">
        <h2 className="text-lg font-semibold text-text-primary">Prediction History</h2>
        <p className="text-sm text-text-muted mt-1">
          {predictions.length} prediction{predictions.length !== 1 ? "s" : ""}
        </p>
      </div>

      <PredictionsTable
        id="agent-predictions-table"
        data={predictions}
        emptyMessage="No predictions from this agent"
      />
    </div>
  );
}
