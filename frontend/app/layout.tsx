import type { Metadata } from "next";
import "./globals.css";
import Nav from "@/components/nav";

export const metadata: Metadata = {
  title: "StarSight — RWA Prediction Intelligence",
  description:
    "Real-time dashboard for AI-driven real-world asset predictions on the Stellar network.",
};

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html lang="en" className="dark">
      <body className="min-h-screen bg-surface-base text-text-primary">
        <a 
          href="#main-content" 
          className="sr-only focus:not-sr-only focus:absolute focus:p-4 focus:bg-surface-base focus:z-50 focus:top-0 focus:left-0"
        >
          Skip to content
        </a>
        <Nav />
        <main id="main-content" className="mx-auto max-w-7xl px-6 py-6">{children}</main>
      </body>
    </html>
  );
}
