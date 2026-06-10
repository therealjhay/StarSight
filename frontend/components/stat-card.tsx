interface StatCardProps {
  label: string;
  value: string | number;
  sub?: string;
}

export default function StatCard({ label, value, sub }: StatCardProps) {
  return (
    <div
      className="border border-border bg-surface-raised px-5 py-4 animate-fade-in"
      id={`stat-${label.toLowerCase().replace(/\s+/g, "-")}`}
    >
      <p className="text-xs uppercase tracking-widest text-text-muted mb-1">
        {label}
      </p>
      <p className="text-2xl font-bold font-mono text-text-primary">{value}</p>
      {sub && (
        <p className="text-xs text-text-muted mt-1">{sub}</p>
      )}
    </div>
  );
}
