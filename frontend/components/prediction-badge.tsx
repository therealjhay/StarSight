const statusStyles: Record<string, string> = {
  Pending: "bg-status-pending/15 text-status-pending border-status-pending/30",
  Resolved: "bg-status-resolved/15 text-status-resolved border-status-resolved/30",
  Scored: "bg-status-scored/15 text-status-scored border-status-scored/30",
};

interface PredictionBadgeProps {
  status: string;
}

export default function PredictionBadge({ status }: PredictionBadgeProps) {
  const style = statusStyles[status] ?? "bg-text-muted/15 text-text-muted border-text-muted/30";

  return (
    <span
      role="status"
      aria-live="polite"
      className={`inline-flex items-center px-3 py-1.5 text-xs font-medium border ${style} min-h-[44px]`}
    >
      {status}
    </span>
  );
}
