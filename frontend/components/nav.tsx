import Link from "next/link";

const links = [
  { href: "/", label: "Dashboard" },
  { href: "/assets", label: "Assets" },
  { href: "/agents", label: "Agents" },
  { href: "/predictions", label: "Predictions" },
] as const;

export default function Nav() {
  return (
    <nav
      id="main-nav"
      className="sticky top-0 z-50 flex items-center gap-8 border-b border-border bg-surface-base/80 backdrop-blur-sm px-6 py-3"
    >
      <Link
        href="/"
        className="text-lg font-bold tracking-wider text-text-primary select-none"
      >
        STAR<span className="text-accent">SIGHT</span>
      </Link>

      <div className="flex items-center gap-6 ml-8">
        {links.map((link) => (
          <Link
            key={link.href}
            href={link.href}
            className="text-sm text-text-muted hover:text-text-primary transition-colors duration-150"
          >
            {link.label}
          </Link>
        ))}
      </div>
    </nav>
  );
}
