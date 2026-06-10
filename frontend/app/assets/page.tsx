import type { Metadata } from "next";
import { getAssets } from "@/lib/api";
import AssetsTable from "@/components/assets-table";
import type { Asset } from "@/lib/types";

export const metadata: Metadata = {
  title: "Assets — StarSight",
  description: "All registered real-world assets on the StarSight network.",
};

export default async function AssetsPage() {
  let assets: Asset[] = [];

  try {
    assets = await getAssets();
  } catch {
    // API unavailable
  }

  return (
    <div>
      <div className="mb-6">
        <h1 className="text-xl font-semibold text-text-primary">Assets</h1>
        <p className="text-sm text-text-muted mt-1">
          All registered real-world assets
        </p>
      </div>

      <AssetsTable data={assets} />
    </div>
  );
}
