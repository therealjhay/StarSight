const statusStyles: Record<string, string> = {
  Pending: "bg-warning/15 text-warning border-warning/30",
  Resolved: "bg-accent/15 text-accent border-accent/30",
  Scored: "bg-success/15 text-success border-success/30",
};

interface PredictionBadgeProps {
  status: string;
}

export default function PredictionBadge({ status }: PredictionBadgeProps) {
  const style = statusStyles[status] ?? "bg-text-muted/15 text-text-muted border-text-muted/30";

  return (
    <span
      className={`inline-block px-2.5 py-0.5 text-xs font-medium border ${style}`}
    >
      {status}
    </span>
  );
}
