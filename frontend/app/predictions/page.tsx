import type { Metadata } from "next";
import PredictionsFeed from "@/components/predictions-feed";

export const metadata: Metadata = {
  title: "Predictions — StarSight",
  description: "Live feed of all predictions on the StarSight network.",
};

export default function PredictionsPage() {
  return <PredictionsFeed />;
}
