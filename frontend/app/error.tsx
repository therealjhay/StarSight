"use client";

import { useEffect } from "react";

export default function GlobalError({
  error,
  reset,
}: {
  error: Error & { digest?: string };
  reset: () => void;
}) {
  useEffect(() => {
    console.error(error);
  }, [error]);

  return (
    <div className="flex flex-col items-center justify-center min-h-[50vh] space-y-4">
      <h2 className="text-xl font-semibold text-text-primary">Something went wrong</h2>
      <p className="text-sm text-text-muted text-center max-w-md">
        We encountered an error loading this data. Please try again or check your connection.
      </p>
      <button
        onClick={() => reset()}
        className="px-4 py-2 bg-accent text-surface-base font-medium rounded hover:opacity-90 transition-opacity min-h-[44px]"
      >
        Try again
      </button>
    </div>
  );
}
