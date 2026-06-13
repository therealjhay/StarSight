"use client";

import { useEffect, useState, type ReactNode } from "react";

interface StatCardProps {
  label: string;
  value: string | number | ReactNode;
  sub?: string;
  delay?: number;
}

export default function StatCard({ label, value, sub, delay = 0 }: StatCardProps) {
  const [isVisible, setIsVisible] = useState(false);
  const [displayValue, setDisplayValue] = useState(0);

  useEffect(() => {
    const timer = setTimeout(() => {
      setIsVisible(true);
      if (typeof value === "number") {
        const target = value;
        const duration = 800;
        const startTime = Date.now();
        const animate = () => {
          const elapsed = Date.now() - startTime;
          const progress = Math.min(elapsed / duration, 1);
          const eased = 1 - Math.pow(1 - progress, 3);
          setDisplayValue(Math.floor(target * eased));
          if (progress < 1) requestAnimationFrame(animate);
        };
        animate();
      }
    }, delay);
    return () => clearTimeout(timer);
  }, [value, delay]);

  const finalValue = typeof value === "number" ? displayValue : value;

  return (
    <div
      className={`border border-border bg-surface-raised px-5 py-4 transition-all duration-300 hover:border-border-hover hover:shadow-lg hover:shadow-accent/10 ${
        isVisible ? "animate-slide-up-fade" : "opacity-0 translate-y-2"
      }`}
      id={`stat-${label.toLowerCase().replace(/\s+/g, "-")}`}
      style={{ animationDelay: `${delay}ms` }}
    >
      <p className="text-xs uppercase tracking-widest text-text-muted mb-1">
        {label}
      </p>
      <p className="text-2xl font-bold font-mono text-text-primary">{finalValue}</p>
      {sub && (
        <p className="text-xs text-text-muted mt-1">{sub}</p>
      )}
    </div>
  );
}
