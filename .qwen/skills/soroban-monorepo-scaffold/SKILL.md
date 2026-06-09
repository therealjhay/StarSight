---
name: soroban-monorepo-scaffold
description: Scaffolding a StarSight-style Soroban monorepo with mixed wasm/host Cargo targets
source: auto-skill
extracted_at: '2026-06-09T13:50:28.400Z'
---

# Soroban Monorepo Scaffold

## Workspace layout

```
monorepo/
├── Cargo.toml              # workspace root listing all crates
├── rust-toolchain.toml     # pinned stable + wasm32-unknown-unknown + rust-src
├── contracts/
│   ├── .cargo/config.toml  # build.target = "wasm32-unknown-unknown"
│   └── <contract>/         # each contract: Cargo.toml + src/lib.rs
├── backend/api/
│   ├── .cargo/config.toml  # host target (no build.target override)
│   └── src/main.rs
├── agent/
│   ├── .cargo/config.toml  # host target (no build.target override)
│   └── src/main.rs
└── frontend/               # Next.js app
```

## Target isolation strategy

- The root `rust-toolchain.toml` declares `wasm32-unknown-unknown` as a globally available target.
- `contracts/.cargo/config.toml` sets `build.target = "wasm32-unknown-unknown"` so all crates under that directory compile to WASM by default.
- `backend/` and `agent/` each have their own `.cargo/config.toml` with **no `build.target` override**, so they compile to the host triple.
- Do NOT put a single `.cargo/config.toml` at the repo root — it would force wasm on all members.

## Contract crate template

```toml
[package]
name = "<contract-name>"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
soroban-sdk = "21.7.0"

[dev-dependencies]
soroban-sdk = { version = "21.7.0", features = ["testutils"] }
```

`src/lib.rs` uses `#![no_std]` and `#[contract]` macro.

## Known pitfalls

1. **`rustup` `rust-src` component conflict** — if the toolchain has a partially installed `rust-src`, installing it again fails with `detected conflict: 'lib/rustlib/src/rust/library/Cargo.lock'`. Fix: `rustup toolchain uninstall stable && rustup toolchain install stable --component rust-src --target wasm32-unknown-unknown`.

2. **`create-next-app@latest` defaults to Next.js 16+** — if Next.js 14 is required, use `npx create-next-app@14` (pin the version explicitly).

3. **Axum 0.7 removed `axum::Server::bind`** — use `tokio::net::TcpListener::bind` + `axum::serve(listener, app)` instead.

4. **Soroban SDK version** — `21.7.0` is compatible with Soroban env `21.x`. Major version bumps may break the `#[contract]` macro.
