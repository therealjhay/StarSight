"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import { useState } from "react";

const links = [
  { href: "/", label: "Dashboard" },
  { href: "/assets", label: "Assets" },
  { href: "/agents", label: "Agents" },
  { href: "/predictions", label: "Predictions" },
] as const;

export default function Nav() {
  const pathname = usePathname();
  const [isOpen, setIsOpen] = useState(false);

  return (
    <nav
      id="main-nav"
      className="sticky top-0 z-50 flex items-center justify-between border-b border-border bg-surface-base/80 backdrop-blur-sm px-6 py-3"
      aria-label="Main navigation"
    >
      <Link
        href="/"
        className="text-lg font-bold tracking-wider text-text-primary select-none"
        aria-label="StarSight home"
      >
        STAR<span className="text-accent">SIGHT</span>
      </Link>

      {/* Desktop Navigation */}
      <div className="hidden sm:flex items-center gap-6">
        {links.map((link) => {
          const isActive = pathname === link.href;
          return (
            <Link
              key={link.href}
              href={link.href}
              className={`text-sm transition-colors duration-normal ${
                isActive
                  ? "text-text-primary font-medium"
                  : "text-text-muted hover:text-text-primary"
              }`}
              aria-current={isActive ? "page" : undefined}
            >
              {link.label}
            </Link>
          );
        })}
      </div>

      {/* Mobile Hamburger */}
      <button
        type="button"
        className="sm:hidden p-2 text-text-muted hover:text-text-primary focus:outline-none focus-visible:ring-2 focus-visible:ring-accent rounded min-h-[44px] min-w-[44px] flex items-center justify-center"
        onClick={() => setIsOpen(!isOpen)}
        aria-expanded={isOpen}
        aria-label="Toggle navigation menu"
      >
        <span className="sr-only">Open menu</span>
        <svg className="w-6 h-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          {isOpen ? (
             <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
          ) : (
             <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 6h16M4 12h16M4 18h16" />
          )}
        </svg>
      </button>

      {/* Mobile Drawer */}
      {isOpen && (
        <div className="sm:hidden absolute top-full left-0 right-0 bg-surface-base border-b border-border shadow-lg p-4 flex flex-col gap-4 animate-slide-up-fade">
          {links.map((link) => {
            const isActive = pathname === link.href;
            return (
              <Link
                key={link.href}
                href={link.href}
                className={`text-base p-2 transition-colors duration-normal rounded ${
                  isActive
                    ? "bg-surface-raised text-text-primary font-medium"
                    : "text-text-muted hover:text-text-primary hover:bg-surface-overlay"
                }`}
                aria-current={isActive ? "page" : undefined}
                onClick={() => setIsOpen(false)}
              >
                {link.label}
              </Link>
            );
          })}
        </div>
      )}
    </nav>
  );
}
