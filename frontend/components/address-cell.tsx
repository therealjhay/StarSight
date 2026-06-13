"use client";

import { useState, useCallback } from "react";

interface AddressCellProps {
  address: string;
}

export default function AddressCell({ address }: AddressCellProps) {
  const [copied, setCopied] = useState(false);

  const truncated =
    address.length > 10
      ? `${address.slice(0, 6)}...${address.slice(-4)}`
      : address;

  const handleCopy = useCallback(async () => {
    try {
      await navigator.clipboard.writeText(address);
      setCopied(true);
      setTimeout(() => setCopied(false), 1500);
    } catch {
      // Clipboard API not available — silently fail
    }
  }, [address]);

  return (
    <button
      onClick={handleCopy}
      className="inline-flex items-center gap-1.5 font-mono text-sm text-text-primary hover:text-accent transition-colors duration-150 group px-3 py-2 rounded min-h-[44px] min-w-[44px]"
      aria-label={`Copy address: ${address}`}
      title={address}
    >
      <span>{truncated}</span>
      <span className="text-text-muted group-hover:text-accent text-xs" aria-hidden="true">
        {copied ? "✓" : "⧉"}
      </span>
      <span className="sr-only">{copied ? "Copied" : "Copy to clipboard"}</span>
    </button>
  );
}
