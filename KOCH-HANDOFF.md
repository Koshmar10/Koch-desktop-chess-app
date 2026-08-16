# Koch → Rebuild Handoff

Context for building the chess app fresh in this repo. Written 2026-08-16 after a full
audit of the previous implementation.

**Read this before writing code here.** It records what was audited, what was decided,
and — importantly — one landmine that already fooled me once.

| | |
|---|---|
| This repo (new) | `/home/petru/storage/Projects/koch/` |
| Reference implementation (old) | `../chess_app/Koch/` — keep read-only until parity |
| Old remote | `git@github.com:Koshmar10/Chess_ai_app.git` |

All file:line references below point into the **old** repo at `../chess_app/Koch/`.

Published audits (fuller detail, same conclusions):
- Frontend: https://claude.ai/code/artifact/c2ac8400-a900-4065-ab96-a559eaaa3f79
- Backend: https://claude.ai/code/artifact/c1fa72c0-0910-48a2-ab06-6ba6a8fef0e7

---

## 1. What the app is

Desktop chess app. Tauri 2 shell, React 19 + TypeScript frontend, Rust backend with an
embedded chess engine and a Stockfish subprocess for analysis. SQLite for game history.
OpenAI-backed chat assistant for position analysis.

Measurements of the old implementation:

| | |
|---|---|
| Frontend | 5,121 lines, 44 files, `src/` |
| Backend | 7,137 lines, 33 files, `src-tauri/src/` |
| Tauri commands | 34 (30 sync, 4 async) |
| Generated TS types | 52, via `ts-rs` |
| Tests | **0**, both sides |
| Lint / format config | **none**, both sides |

Feature surface to reach parity with: Home, PvE vs Stockfish (with clocks + Elo),
Analyzer (multipv, threat arrows, influence tinting, AI chat), Game History (with
chess.com sync + PGN import), Puzzles, Settings.

---

## 2. Decisions taken (2026-08-16)

The user is mid-internship, has learned real dev process (feature branches, tickets,
tests, CI), and wants to apply it from commit one. They chose:

1. **New repo, port the engine.** Fresh shell with workspace + CI + tests from the
   start. Lift `engine/` across mostly as-is, then write the tests it never had.
   Rebuild persistence, config, command layer, and the whole frontend properly.
2. **GitHub Issues + Projects** for ticket tracking.
   ⚠️ `gh` CLI is **not installed** on this machine — needs installing before issues can
   be created/labelled/linked directly.
3. **Rust-native AI.** Keep the `ai/` design, drop the Python agent-server, the
   PyInstaller binary, and the ChromaDB store.
4. **Engine tested hard, rest pragmatic.** Perft/FEN/legality tests with real coverage
   on the engine crate; integration tests on DB + commands; frontend gets lint +
   typecheck and tests only for pure logic. CI runs fmt/clippy/test on every PR.
5. **Own the UCI layer, on tokio.** Drop the `stockfish` crate; write the Stockfish
   communication layer in-house, async. See §5 for the design and the reasoning.
6. **AI abstraction deferred.** Call OpenAI directly for now, with one discipline
   attached — no provider types cross out of `ai/`. See §5.

### Why a rewrite is justified here

Worth preserving the reasoning, because "it feels messy" was *not* the good reason:

- **Frontend**: the mess was shallow and concentrated. Verdict was *refactor* — later
  superseded by the decision to rebuild everything. Its findings survive as a
  "don't repeat these" list (§6), not a work plan.
- **Backend**: the code was genuinely better than the frontend, yet the case for
  restarting was stronger — because what you gain is **a tested engine** and **a build
  that runs on a machine that isn't yours**. Both real, neither aesthetic.

---

## 3. ⚠️ Landmine: the eval perspective flag

**I got this wrong on the first pass. Do not repeat it.**

`EvalScore` in the old bindings looks like this:

```ts
{ centipawns: number | null, mate_in: number | null, is_from_white_perspective: boolean }
```

In `../chess_app/Koch/src-tauri/src/analyzer/analyzer.rs:532`:

```rust
centipawns: Some(score_value * color_multiplier),   // ← already White-relative
is_from_white_perspective: color_multiplier == 1,   // ← "was White to move"
```

The value **is** normalized into White's frame. The flag is set from the
*pre*-normalization condition, so it reads `false` whenever Black is to move despite
the value always being White-relative.

- Reading `centipawns` directly is **correct**.
- Honoring the flag **double-flips the sign** and introduces a bug.
- In this repo: **delete the field**, or rename it `side_to_move_was_white`.

Related real bug carried over from the old frontend: `Analyzer.tsx:410–444`
`updateSuggestion` re-ranked the multipv lines by highest score. Since all scores are
White-relative, for Black to move this picks the *worst* line and the suggestion arrow
points at a blunder. Correct behavior is to take multipv index 1 — Stockfish already
orders best-first. The old eval bar got this right via `getFirstPvLine`; the suggestion
path did not.

---

## 4. The three backend concerns

The backend is really three features with very different shapes. Keeping them separate
is the main structural decision:

| Layer | What it is | Dependency stance |
|---|---|---|
| **Chess logic** | Pure, in-process, deterministic | Own it. `koch-engine` crate, no Tauri, heavily tested. |
| **Stockfish** | Stateful subprocess, streaming, latency-sensitive | **Own it** — see §5. Drop the `stockfish` crate. |
| **AI** | Stateless HTTP request/response | Own the transport, but thin. Lowest-value layer to hand-roll. |

Weight the effort accordingly: **UCI is a milestone, the AI layer is a ticket.** Splitting
them evenly means spending a week rebuilding an HTTP client that was never the problem.

---

## 5. Owning UCI and AI — reasoning and design

### Why drop the `stockfish` crate

The crate (v0.2.11, 707 lines, zero deps) has a reasonable API — `go()`, `go_for()`,
`EngineOutput`. **The old analyzer bypassed almost all of it**, using raw `uci_send()` +
`read_line()` and parsing `multipv` / `depth` / `score` / `pv` tokens by hand inside the
thread loop (`analyzer.rs:500–527`).

That wasn't laziness, it was a model mismatch: the crate is **blocking and single-shot**
(ask for one best move, block, get a result), while the analyzer needs **streaming,
multipv, interruptible** search where a new position supersedes the in-flight one. Those
don't reconcile.

Concrete hazard in the escape hatch: `read_line()` is typed `-> String`, not
`-> io::Result<String>`. If the engine process dies you get empty strings forever and
the analyzer thread spins silently. Nothing panics, so the `catch_unwind` guard doesn't
help.

So the argument is not "more control" in the abstract — the hard part was already
hand-written and badly placed, and the dependency only held process management.

### UCI layer design — three pieces

The split exists so the parsing becomes testable. Right now it can't be tested at all
because it's welded inside a thread loop.

- **`UciProcess`** — spawn, write a line, read lines onto a channel. I/O only, thin.
- **`UciParser`** — **pure**: `&str → InfoLine`. No I/O, no state. Record real Stockfish
  output into fixture files once; test every parse path with no subprocess involved.
  This is where the test coverage lives.
- **`Engine`** — typed commands and session state on top of the other two.

**Runtime: tokio** (decided). `tokio::process::Child` + `BufReader::next_line()` +
broadcast channel; cancellation via `select!` and a `CancellationToken` replaces the old
`try_recv` polling loop. Chosen because commands go async in M3, and mixing sync `mpsc`
with async commands is where deadlocks live.

Budget ~400–600 lines including tests. UCI is a stable, specified protocol — write-once.

Note: the old `engine/uci.rs` is **not** a protocol layer — it's algebraic ↔ coordinate
move encode/decode. That belongs in `koch-engine`, not here.

### AI layer — deferred, and how to keep it cheap

`rig` was already removed from the old `Cargo.toml`; that code was hand-rolled `reqwest`
+ `serde_json::json!` against OpenAI. So owning it is the status quo, not a change. What
was missing was structure, not ownership: `MODEL = "gpt-4.1"`, `MAX_ITERATIONS = 16`,
and the tool schemas were all inline constants and raw JSON blobs — nothing configurable,
nothing testable without network.

The abstraction decision is **deliberately deferred**. Deciding it now means designing a
trait against a single implementation, which reliably encodes that implementation's
quirks as if they were universal.

**The rule that keeps it deferrable: don't write the trait, write the boundary.** Call
OpenAI directly, but every signature the rest of the app touches must be in domain terms
— `analyze_position(ctx) -> Analysis`, never `chat_completion(msgs) -> OpenAiResponse`.
No provider JSON shapes cross out of `ai/`. If that holds, extracting a trait later is
mechanical. If it doesn't, no amount of upfront design saves you.

Do be deliberate about two things, because they are where hand-rolled clients break:
**retry/backoff on 429 and 5xx**, and **validating tool-call arguments before dispatch**.

Non-goal: a general-purpose LLM framework.

---

## 6. Carry / Rebuild / Drop

### Carry across (port, don't redesign)

- **`engine/`** — ~2,700 lines of pure chess logic, no Tauri/DB/IO dependencies.
  `board.rs` (1,331), `move_gen.rs` (588), `fen.rs` (380), plus `capture.rs`,
  `quiet.rs`, `simulate.rs`, `piece.rs`, `san.rs`, `uci.rs`, `serializer.rs`.
  → Becomes a standalone `koch-engine` crate with no Tauri dep.
- **The `ts-rs` boundary.** 52 types generated from Rust. Keep the pattern exactly —
  it's the reason frontend and backend can't drift, and most projects never set it up.
- **The analyzer thread design** (`analyzer.rs:306–560`). Bounded `sync_channel`, a
  single reused Stockfish process, stale-request coalescing via `try_recv` drain, and a
  `catch_unwind` guard around the thread body. Genuinely well thought out.
- **The chessboard React component.** Already properly decomposed:
  `useChessInteraction` / `useInfluenceTint` / `useCheckSquares` hooks, plus
  `PieceLayer` / `ArrowLayer` / `BoardSquare`. Optimistic piece movement works. This is
  the hardest component in the app and the cleanest.
- **The visual language.** Warm-brown palette (`#8B6F47` primary, `#C9A875` dark
  primary, `#1A1310` ground, `#F4EEDD` foreground). Design decisions are settled and
  worth porting — the *implementation* was a mess, see §8.
- **Parameterized SQL.** Every old query used `?1` / `params![]`. No injection anywhere.
  Keep that discipline.

### Rebuild properly

- **Persistence** — app-data dir from Tauri's path resolver, one pooled connection,
  versioned migrations (`user_version` pragma).
- **Config** — typed struct with serde, not `HashMap<String,String>`.
- **Resource loading** — bundle `openings.json` (233 KB) via `include_str!` or as a
  Tauri resource.
- **Errors** — one crate-wide enum via `thiserror`; commands return `Result<T, AppError>`.
- **Command layer** — async by default, blocking work off the main thread.
- **Secrets** — OS keychain or env var. Never a file the repo knows about.

### Do not bring over

- `database/save_game.rs` — 0 bytes.
- `bin/test_stockfish.rs` — scratch binary.
- `destroy_database()` — drops the games table, called by nothing.
- Committed binaries: `dist_new/`, `src-tauri/binaries/`, the ChromaDB store.
- `koch.config` (see §5).
- The Python agent-server + PyInstaller binary (decision #3).
- `src/components/mock.tsx` (76 lines, imported by nothing), `package.json (edit)`,
  `chat_history.txt`, `ECO Listing.html`.

---

## 7. Blockers found in the old repo — do not recreate these

1. **Absolute dev paths compiled into the binary.** `server.rs:272`, `create.rs:242`,
   `integrations.rs:37` all hardcoded
   `/home/petru/storage/Projects/chess_app/Koch/src-tauri/src/…`. The openings load
   ended in `.ok()`, so on any other machine ECO detection silently returned `None`.
   **The app only ran on the dev machine.**
2. **`Connection::open("chess.db")` in 10 places**, relative path. Resolves against
   CWD, which differs between `tauri dev`, a bundled `.deb`, and a desktop launcher —
   users got a different empty DB depending on how they launched. No shared connection.
   Settings had the same problem, writing to `../koch.config`.
3. **OpenAI key wrote into a git-tracked file.** Read from
   `settings.map["OpenAiApiKey"]` (`message.rs:82`); `Settings::save()` wrote the whole
   map to `../koch.config` (`server.rs:156`), which **was tracked**. Nothing leaked
   (the committed file was checked, no key present) but one settings-panel write would
   have done it. Also: the last line of `.gitignore` was an absolute path, which git
   treats as repo-relative and therefore matched nothing.
4. **`UNIQUE(content)` on the `messages` table** (`create.rs:51`) — global, not scoped
   to a chat. Two identical assistant replies, or a user typing "why?" twice, and the
   second insert failed outright.
5. **89 `unwrap`/`expect`** (33 in the DB layer, 14 each in `lib.rs` and
   `controller.rs`). Worse than the count: `fetch_game` logged a DB error and returned
   `AnalyzerController::default()`, so **a database failure was indistinguishable from
   an empty board** in the UI.
6. **30 of 34 commands were synchronous** — PGN parsing, history queries, and every DB
   write ran on Tauri's main thread while holding a `Mutex`.
7. **No migrations.** Three `CREATE TABLE IF NOT EXISTS` statements, no versioning. The
   git log said "updated database schema" — for existing users those columns never
   appeared.

---

## 8. Frontend anti-patterns to avoid repeating

- **No data layer.** 33 `invoke` calls scattered across 11 components, so each
  reinvented loading, cancellation, and cache invalidation. → This repo gets `src/api/`,
  one typed module per backend domain; no component imports `@tauri-apps/api/core`.
- **God components.** `Analyzer.tsx` = 700 lines / 27 `useState` / 8 `useEffect`.
  `Pve.tsx` = 502 lines / 18 `useState`, with the same 8-line state fan-out block
  repeated **six times**. → One reducer per domain, not N independent `useState`.
- **Screens as a `switch`, not routes.** Leaving the Analyzer destroyed all 27 pieces of
  state; returning re-fetched the game, reloaded chat, and restarted the engine from
  ply −1. No back navigation, no deep links.
- **Two theme systems, wrong one live.** Tailwind v3 ran via PostCSS off hardcoded hex
  in `tailwind.config.js`, while `App.css` held a v4 `@theme` block + `:root`/`.dark`
  CSS vars that v3 can't read (all dead). `darkMode: "class"` was configured but `.dark`
  was never applied — **0** `dark:` variants against ~210 hardcoded `-dark` class names
  like `text-foreground-dark`. The light theme was fully specified and unreachable.
- **Effects used as actions.** The LLM request fired from an effect watching
  `[chatHistory]`; the engine-move timer likewise. Sending a request is an action, not a
  consequence of an array changing.
- **Flags where tokens belong.** `fetchIndex` guarded on `isFetching` *state* (stale
  within the tick), so held arrow keys dropped navigation instead of superseding it —
  bypassing the `latestTokenRef` mechanism that was already there and correct.
- **Cleanup returned from event handlers.** `Pve.tsx:89` and `:215` ended with
  `return () => clearTimeout(t)` from plain async handlers. React never calls those.
- **Side effects inside state updaters.** The PvE clock called `endGame()` from within
  a `setState` updater (StrictMode double-invokes → game ends twice), and decremented a
  fixed `-100` rather than reading wall time, so it drifted.
- **Hook in a render helper.** `AnalyzerSettings.tsx:27` called `useState` inside
  `loadAnalyzerModeContent()`, invoked as a plain function in JSX. Survived only because
  the call was unconditional.
- **61 deep relative imports** of `../../../src-tauri/bindings/…`, no path alias.
- Board geometry as a magic number per screen (`squareSize = 72` in Analyzer, `85` in
  PvE) with layout heights derived as `squareSize * 9.5`.
- `index.html` pulled Roboto Mono from **Google Fonts over the network** in a desktop
  app; title was still "Tauri + React + Typescript"; `<body class="f">` typo.
- `main.tsx` raced three separate `show_main_window` calls.

Two of these (the hook violation, unused locals) are caught free by a standard ESLint
React preset — which is why lint lands in M0, before any product code.

---

## 9. Milestone plan

Ordered so each ends with something demonstrably working and hands over as a ticket.

- **M0 — Repo skeleton with rails on.** Cargo workspace, `rustfmt` + `clippy` in CI,
  ESLint + Prettier + `tsc`, branch protection on `main`, PR template, conventional
  commits. Empty crates, green pipeline. *No product code* — the habits should have
  nothing to fight.
- **M1 — Port the engine, then test it.** `koch-engine` crate, no Tauri dep. Then perft
  depths 1–4 from start position and Kiwipete, FEN round-trip, castling rights, en
  passant, promotion, pin legality. **Expect to find real bugs.** Highest-value
  milestone.
- **M2 — Persistence done right.** App-data dir, pooled connection, versioned
  migrations, typed `Settings` with serde, secrets out of config entirely.
- **M3 — Command layer with real errors.** One `thiserror` enum,
  `Result<T, AppError>`, async by default. ts-rs bindings regenerate free.
- **M4 — UCI layer, in-house.** The `UciProcess` / `UciParser` / `Engine` split from §5,
  on tokio. Parser tested against recorded Stockfish fixtures. Port the old thread
  design's *ideas* — bounded channel, one reused process, stale-request coalescing —
  but not its code. Fix/remove the eval flag (§3). Make the Stockfish path configurable
  with discovery instead of hardcoded `/usr/bin/stockfish`; decide bundled vs
  prerequisite. **Second-largest milestone after M1.**
- **M5 — Frontend shell.** Router, `src/api/` layer, single theme system with real
  `dark:` variants, path aliases. Port the chessboard component.
- **M6 — Feature parity, screen by screen.** PvE → Analyzer → History → Puzzles →
  Settings. One screen per ticket.
- **M7 — AI assistant.** Rust-native, direct OpenAI calls, domain-term boundary only —
  no trait yet (§5). Model name and iteration cap into config (old code hardcoded
  `gpt-4.1` and `MAX_ITERATIONS = 16` in `ai/agent.rs`). Typed tool registry instead of
  `json!` blobs. Retry/backoff on 429 and 5xx. Key in the keychain. **Sized as a ticket,
  not a milestone.**

---

## 10. Working agreement

- Break work into GitHub Issues; one issue per shippable slice, milestone-labeled.
- Feature branch per ticket, PR into `main`, CI green before merge.
- Conventional commits.
- Engine changes require tests. Frontend changes require lint + typecheck to pass.
- The user is learning process deliberately — prefer explaining *why* a practice exists
  over just applying it, and don't skip steps to save time.

## 11. Immediate next actions

1. Install `gh` CLI (blocker for issue creation).
2. `git init` here; create the GitHub repo.
3. Land M0, then open the M1 ticket set.
4. Keep `../chess_app/Koch/` read-only as the reference implementation until M6
   completes.
