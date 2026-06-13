import type { Metadata } from "next";
import { getAgents } from "@/lib/api";
import AgentsTable from "@/components/agents-table";
import type { AgentScore } from "@/lib/types";

export const metadata: Metadata = {
  title: "Agents — StarSight",
  description: "Agent leaderboard ranked by prediction accuracy.",
};

export default async function AgentsPage() {
  const agents: AgentScore[] = await getAgents();

  return (
    <div>
      <div className="mb-6">
        <h1 className="text-xl font-semibold text-text-primary">
          Agent Leaderboard
        </h1>
        <p className="text-sm text-text-muted mt-1">
          Ranked by prediction accuracy
        </p>
      </div>

      <AgentsTable data={agents} />
    </div>
  );
}
