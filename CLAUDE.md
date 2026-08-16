# Koch

Desktop chess app — Tauri 2 + React 19 + Rust, with an embedded engine, a Stockfish
subprocess for analysis, SQLite history, and an AI position assistant.

This is a **deliberate rebuild** of a previous implementation, started 2026-08-16.

## Read first

**[KOCH-HANDOFF.md](KOCH-HANDOFF.md)** — audit of the old codebase, the decisions taken,
the carry/rebuild/drop triage, and the M0–M7 milestone plan. Read it before writing code
or proposing architecture here. In particular §3 records a sign-convention landmine in
the eval pipeline that has already caused one wrong fix.

The previous implementation is at `../chess_app/Koch/` — treat it as **read-only
reference** until feature parity is reached. All file:line references in the handoff
point there.

## How work is organised

- GitHub Issues + Projects, one issue per shippable slice, milestone-labeled.
- Feature branch per ticket → PR into `main` → CI green before merge. No direct pushes.
- Conventional commits.
- Engine changes require tests. Frontend changes require lint + typecheck to pass.

## Quality bar

- `koch-engine` is tested hard: perft, FEN round-trip, castling, en passant, promotion,
  pin legality. This crate has no Tauri dependency and must stay that way.
- Persistence and command layers get integration tests.
- Frontend gets ESLint + `tsc`; unit tests only for pure logic.
- CI runs `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test`, `eslint`,
  and `tsc --noEmit` on every PR.

## Notes for Claude

- The user is learning software process deliberately during an internship. Explain *why*
  a practice exists rather than just applying it, and don't skip steps to save time.
- Never write secrets into tracked files. The old repo nearly leaked an OpenAI key this
  way — keys go in the OS keychain or an env var.
- No absolute paths in source. The old repo compiled in `/home/petru/...` paths and only
  ran on this machine.
