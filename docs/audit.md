# StarSight Frontend — Technical Audit Report

**Date:** 2026-06-13
**Project:** StarSight — RWA Prediction Intelligence (Stellar/Soroban)

---

## Audit Health Score

| # | Dimension | Score | Key Finding |
|---|-----------|-------|-------------|
| 1 | Accessibility | **2/4** | Missing landmarks, improper heading hierarchy, keyboard nav gaps |
| 2 | Performance | **2/4** | No lazy loading, missing memoization, layout animations on scroll |
| 3 | Theming | **3/4** | Good token usage, minor hard-coded values in status indicators |
| 4 | Responsive Design | **2/4** | Tables overflow on mobile, touch targets too small in places |
| 5 | Anti-Patterns | **2/4** | Multiple AI tells: card grids, generic dashboard layout, safe colors |

**Total: 11/20 — Acceptable (significant work needed)**

---

## Anti-Patterns Verdict

**FAIL** — This looks AI-generated. Specific tells detected:

1. **Hero metric dashboard layout** (StatCard grid on home page) — the classic "3 metrics + card grid" template
2. **Generic dark mode with blue accent** — `#0a0a0f` + `#3b82f6` is the default AI palette
3. **Uniform card grids** — identical StatCard, DataTable styling repeated across pages
4. **Safe, forgettable typography** — Inter + JetBrains Mono with flat hierarchy
5. **Glassmorphism-lite** — `backdrop-blur-sm` on nav, `bg-surface-base/80` without purpose
6. **Border-border everywhere** — monotonous containerization, nested cards
7. **No visual personality** — indistinguishable from any generic dashboard template

---

## Executive Summary

- **P0 (Blocking)**: 3 issues
- **P1 (Major)**: 8 issues
- **P2 (Minor)**: 14 issues
- **P3 (Polish)**: 6 issues

### Top 5 Critical Issues

1. **[P0] Tables completely unusable on mobile** — horizontal overflow, no responsive strategy
2. **[P0] Missing landmarks & semantic structure** — no `<main>`, `<nav>`, `<section>`, heading hierarchy broken
3. **[P0] Keyboard navigation broken in DataTable** — `aria-sort` on `<button>` invalid, no arrow key nav
4. **[P1] No loading/error/empty states on data pages** — only Assets page has empty state
5. **[P1] Touch targets < 44px** — AddressCell, sort buttons, status indicators too small

---

## Detailed Findings by Severity

### P0 — Blocking

**[P0] Tables overflow on mobile (horizontal scroll on viewport)**
- **Location**: `components/data-table.tsx`, `components/predictions-feed.tsx`
- **Category**: Responsive Design
- **Impact**: Data completely unreadable on phones; users must scroll horizontally losing context
- **Standard**: WCAG 1.4.10 Reflow
- **Recommendation**: Implement responsive table pattern — card view on mobile, table on desktop. Use container queries.

**[P0] Missing semantic landmarks and heading hierarchy**
- **Location**: `app/layout.tsx:17-21`, `app/page.tsx:37-42`, all page.tsx files
- **Category**: Accessibility
- **Impact**: Screen reader users cannot navigate; no skip links, no page structure
- **Standard**: WCAG 1.3.1, 2.4.1
- **Recommendation**: Add `<main>`, proper `<h1>` per page, `<nav>` wrapper, skip-to-content link

**[P0] Invalid ARIA in DataTable sort buttons**
- **Location**: `components/data-table.tsx:95-103`
- **Category**: Accessibility
- **Impact**: `aria-sort` not supported on `role="button"`; screen readers announce incorrectly
- **Standard**: ARIA 1.2, WCAG 4.1.2
- **Recommendation**: Use native `<th>` with `aria-sort` or proper `role="columnheader"` pattern

---

### P1 — Major

**[P1] No loading skeletons — only spinners/text**
- **Location**: `components/predictions-feed.tsx:108-115`, `app/assets/page.tsx`, `app/agents/page.tsx`
- **Category**: Performance / UX
- **Impact**: Layout shift on load; perceived slowness; no progressive disclosure
- **Recommendation**: Add skeleton rows matching table structure

**[P1] Touch targets below 44×44px minimum**
- **Location**: `components/address-cell.tsx:30`, `data-table.tsx:98`, `prediction-badge.tsx:18`
- **Category**: Accessibility / Responsive
- **Impact**: Difficult to tap on mobile; fails WCAG 2.5.5
- **Recommendation**: Increase padding to `px-3 py-2.5` minimum; use `min-h-[44px] min-w-[44px]`

**[P1] No error boundaries or retry UI**
- **Location**: All async pages (`app/page.tsx`, `app/assets/page.tsx`, etc.)
- **Category**: UX / Anti-Pattern
- **Impact**: Silent failures; users see blank/zero state with no recovery
- **Recommendation**: Add error state with retry button and user-friendly message

**[P1] WebSocket URL hardcoded to localhost**
- **Location**: `components/predictions-feed.tsx:8`
- **Category**: Performance / Config
- **Impact**: Breaks in production; no env config
- **Recommendation**: Use `process.env.NEXT_PUBLIC_WS_URL` with fallback

**[P1] PredictionsFeed duplicates table markup instead of reusing DataTable**
- **Location**: `components/predictions-feed.tsx:117-152`
- **Category**: Anti-Pattern / Maintainability
- **Impact**: Code duplication; inconsistent behavior; double maintenance burden
- **Recommendation**: Refactor to use `PredictionsTable` component with live data prop

**[P1] No virtualization for large datasets**
- **Location**: `components/data-table.tsx`, `components/predictions-feed.tsx` (caps at 200)
- **Category**: Performance
- **Impact**: DOM bloat with 200+ rows; jank on scroll/sort
- **Recommendation**: Add `@tanstack/react-virtual` or similar for lists > 50 rows

**[P1] AddressCell copy button has no visible focus state distinct from hover**
- **Location**: `components/address-cell.tsx:30`
- **Category**: Accessibility
- **Impact**: Keyboard users can't see focus; relies on browser default
- **Recommendation**: Add explicit `focus-visible:ring-2 focus-visible:ring-accent`

---

### P2 — Minor

**[P2] Hard-coded status colors in PredictionBadge**
- **Location**: `components/prediction-badge.tsx:1-5`
- **Category**: Theming
- **Impact**: Bypasses design tokens; won't adapt to brand changes
- **Recommendation**: Map status → token (e.g., `bg-warning/15` → `bg-status-pending`)

**[P2] Duplicate empty state markup across components**
- **Location**: `data-table.tsx:64-71`, `predictions-feed.tsx:112-115`
- **Category**: Anti-Pattern
- **Recommendation**: Extract `EmptyState` component

**[P2] `animate-fade-in` on every table row causes stagger but no orchestration**
- **Location**: `data-table.tsx:75,121`, `predictions-feed.tsx:135`
- **Category**: Performance / Animation
- **Impact**: 200 rows × animation = layout thrash; no reduced-motion guard on class
- **Recommendation**: Remove per-row animation; use single container fade + stagger via CSS

**[P2] Date formatting inconsistent (ISO slice vs locale)**
- **Location**: `predictions-table.tsx:96-100`, `predictions-feed.tsx:146-147`, `agents-table.tsx:20`
- **Category**: UX / Consistency
- **Recommendation**: Centralize in `lib/format.ts` with `Intl.DateTimeFormat`

**[P2] No keyboard shortcut to copy address (AddressCell)**
- **Location**: `components/address-cell.tsx`
- **Category**: Accessibility
- **Recommendation**: Add `onKeyDown` for Enter/Space; show tooltip on focus

**[P2] StatCard uses `animate-fade-in` but no entrance choreography**
- **Location**: `components/stat-card.tsx:10`
- **Category**: Animation
- **Impact**: All 3 cards pop in simultaneously; no stagger
- **Recommendation**: Stagger with CSS `animation-delay` via `nth-child`

**[P2] Navigation has no mobile hamburger — wraps/overflows**
- **Location**: `components/nav.tsx:19-48`
- **Category**: Responsive
- **Impact**: Links stack awkwardly or overflow on < 640px
- **Recommendation**: Add mobile drawer pattern

**[P2] `getPredictionsByAsset` fetches ALL predictions then filters client-side**
- **Location**: `lib/api.ts:60-64`
- **Category**: Performance
- **Impact**: O(N) transfer + filter; scales poorly
- **Recommendation**: Add server-side endpoint `/predictions/asset/:id`

**[P2] No `will-change` or `contain` on animated elements**
- **Location**: `globals.css`, components using `animate-fade-in`
- **Category**: Performance
- **Recommendation**: Add `.animate-fade-in { will-change: opacity; contain: layout }`

**[P2] Redundant "Dashboard" label in nav + page heading**
- **Location**: `nav.tsx:7`, `page.tsx:38`
- **Category**: UX Writing
- **Recommendation**: Remove one; page heading should be contextual

**[P2] Emoji in streak display (`🔥`) — not accessible**
- **Location**: `agents-table.tsx:54`, `agents/[address]/page.tsx:58`
- **Category**: Accessibility
- **Impact**: Screen readers announce "fire"; no text alternative
- **Recommendation**: Use `aria-label` or text badge "Streak: 5"

**[P2] ESLint error: Comments inside JSX children should be in braces**
- **Location**: `components/data-table.tsx:102-103`
- **Category**: Code Quality
- **Impact**: Build breaks on `npm run lint`; comment text nodes render as text
- **Recommendation**: Wrap in `{/* ... */}` or remove

**[P2] ESLint warning: `aria-sort` not supported on button role**
- **Location**: `components/data-table.tsx:100`
- **Category**: Accessibility / Code Quality
- **Impact**: Disabled lint rule; screen reader mismatch
- **Recommendation**: Restructure sort controls to use `role="columnheader"` with `aria-sort`

---

### P3 — Polish

**[P3] Scrollbar styling only targets WebKit**
- **Location**: `globals.css:22-37`
- **Recommendation**: Add `scrollbar-width: thin; scrollbar-color` for Firefox

**[P3] Transition duration hardcoded as `duration-150`, `duration-100` in multiple places**
- **Recommendation**: Define `--duration-fast`, `--duration-normal` tokens

**[P3] No page transition animations between routes**
- **Recommendation**: Add framer-motion `AnimatePresence` layout transitions

**[P3] Favicon is generic Next.js default**
- **Recommendation**: Custom favicon matching brand

**[P3] No meta theme-color for mobile browser chrome**
- **Location**: `app/layout.tsx`
- **Recommendation**: Add `<meta name="theme-color" content="#0a0a0f">`

**[P3] Agent detail page shows raw timestamp without relative time**
- **Location**: `app/agents/[address]/page.tsx:62-65`
- **Recommendation**: Show "2h ago", "3d ago" with full timestamp on hover

---

## Patterns & Systemic Issues

| Pattern | Occurrences | Fix |
|---------|-------------|-----|
| Hard-coded spacing/colors in status indicators | 4 files | Design tokens for all semantic states |
| Duplicate empty/error/loading state markup | 6+ locations | Extract `DataDisplay` wrapper component |
| Tables not responsive | 3 table components | Single responsive table pattern |
| No virtualization for lists | 2 feeds | Add virtualization library |
| Inconsistent date formatting | 5 locations | Centralize in `lib/format.ts` |
| `animate-fade-in` overused without choreography | 8 usages | Remove per-item; orchestrate at container |

---

## Positive Findings

✅ **TypeScript strict mode** — types well-defined in `lib/types.ts`
✅ **API layer separation** — clean `lib/api.ts` with proper error handling
✅ **WebSocket reconnection logic** — automatic reconnect with backoff
✅ **Address copy UX** — truncated display + copy button + tooltip + confirmation
✅ **Sortable columns with keyboard support** — `DataTable` has solid foundation
✅ **Reduced motion media query** — present in `globals.css:40-49`
✅ **CSS variable theming** — Tailwind config uses semantic tokens consistently
✅ **Component composition** — `DataTable`, `PredictionsTable`, `AddressCell` reusable

---

## Recommended Fix Priority

### Phase 1 — Accessibility & Structure (P0)
1. `/clarify` — Fix semantic landmarks, heading hierarchy, skip links
2. `/clarify` — Fix invalid `aria-sort` on button, JSX comment ESLint error
3. `/adapt` — Mobile responsive tables (card view < 768px)

### Phase 2 — Polish & UX (P1)
4. `/polish` — Add loading skeletons, error/empty states with retry
5. `/adapt` — Increase touch targets to 44×44px minimum
6. `/distill` — Unify PredictionsFeed → PredictionsTable; extract EmptyState
7. `/optimize` — WS URL env var; virtualization for large lists

### Phase 3 — Refinement (P2)
8. `/colorize` — Design tokens for status indicators
9. `/animate` — Orchestrated entrance animations, fix fade-in overuse
10. `/clarify` — Centralized date formatting, accessible emoji
11. `/adapt` — Mobile navigation drawer

### Phase 4 — Polish (P3)
12. `/polish` — Firefox scrollbar, transition tokens, theme-color, page transitions

---

**Next step:** Run each phase in sequence. Re-run `/audit` after all fixes to see score improvement.
