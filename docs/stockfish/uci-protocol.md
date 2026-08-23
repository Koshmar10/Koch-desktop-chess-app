# UCI protocol — as Stockfish 17.1 speaks it

Sources: the canonical UCI protocol specification, authored by Stefan Meyer-Kahlen,
[reproduced here](https://gist.github.com/DOBRO/2592c6dad754ba67e6dcaec8c90165bf)
("the spec" below); the official
[Stockfish UCI & Commands docs](https://official-stockfish.github.io/docs/stockfish-wiki/UCI-Protocol-and-Stockfish-Commands.html)
("the SF docs" below); and the local `/usr/bin/stockfish` binary (Stockfish 17.1),
probed directly — raw output in [fixtures/](fixtures/).

## Transport and lifecycle rules

- Plain **stdin/stdout, line-based, text**. Every command ends `\n`. (Spec.)
- The engine **must keep reading and processing stdin even while searching** — this is
  how `stop` and `isready` interrupt a `go` in progress. A blocking, single-shot read
  loop can't honor this, which is the exact mismatch KOCH-HANDOFF.md §5 gives as the
  reason the `stockfish` crate didn't fit the analyzer's needs. (Spec.)
- Unknown commands and unknown tokens should be **silently ignored**, not treated as
  errors. (Spec.) `UciParser` should default to ignoring unrecognized `info` subfields
  rather than failing to parse the whole line.
- Handshake, observed live against 17.1 (`docs/stockfish/fixtures/handshake-sf17.1.txt`):

  ```
  → uci
  ← Stockfish 17.1 by the Stockfish developers (see AUTHORS file)
  ← id name Stockfish 17.1
  ← id author the Stockfish developers (see AUTHORS file)
  ← (blank line)
  ← option name Debug Log File type string default <empty>
  ← option name NumaPolicy type string default auto
  ← ... (one line per option, all of them, see § Options below)
  ← uciok
  ```

  The banner line before `id name` (`Stockfish 17.1 by the Stockfish developers...`) is
  **not part of the UCI protocol** — it's printed unconditionally at process start,
  before any command is even read. `UciParser` needs to tolerate/ignore it rather than
  choke on it as a malformed `id` or `info` line.
- `isready` → `readyok` must be answered **even mid-search** — it's the synchronization
  primitive, not a "is the engine alive" check gated on search completing. (Spec, SF
  docs.)
- **Every `go` must eventually produce exactly one `bestmove`.** Spec, verbatim: *"this
  command must always be sent if the engine stops searching, also in pondering mode if
  there is a 'stop' command, so for every 'go' command a 'bestmove' command is
  needed!"* This is the invariant `Engine` can rely on to know a search has ended —
  there is no separate "search finished" signal, `bestmove` *is* that signal.

## GUI → engine commands

| Command | Effect |
|---|---|
| `uci` | Switch to UCI mode. Engine replies with `id`, all `option` lines, then `uciok`. |
| `isready` | Ping. Engine replies `readyok` once idle-safe to accept the next command, even during search. |
| `setoption name <id> [value <x>]` | Set an engine option (case-insensitive name). `button`-type options take no `value`. |
| `ucinewgame` | Tell the engine a new game is starting (not just a new position) — it should clear game-specific state (hash, history heuristics). SF docs: should be followed by `isready` before the next `position`/`go`. |
| `position [fen <fenstring> \| startpos] moves <m1> ... <mN>` | Set up a position, then play `moves` from it. **Always include the full move list from `startpos`/the given FEN**, not just the latest move — the engine uses it for repetition/50-move detection, it doesn't retain board state between `position` calls itself. (Spec, SF docs.) |
| `go [params]` | Start searching the current position. See subcommands below. Defaults to depth 245 if no params given at all (SF docs) — in practice `Engine` should never send a bare `go`. |
| `stop` | Stop calculating as soon as possible, then send `bestmove`. |
| `ponderhit` | The move the engine was pondering on was actually played; switch from pondering to a normal timed search on it. |
| `quit` | Terminate the process. |

### `go` subcommands (spec)

| Subcommand | Meaning |
|---|---|
| `searchmoves <m1> ... <mN>` | Restrict the search to only these moves at the root. |
| `ponder` | Start a search on the move the engine is pondering on; the engine must **not** decide the search is "done" on its own — only `ponderhit`/`stop` ends it, even if it internally reaches what looks like a decisive line. |
| `wtime <x>` / `btime <x>` | Milliseconds left on White's/Black's clock. |
| `winc <x>` / `binc <x>` | Increment per move, in ms. |
| `movestogo <x>` | Moves left until the next time control; if omitted, assume sudden death. |
| `depth <x>` | Search only this many plies. |
| `nodes <x>` | Search (approximately) this many nodes then stop. |
| `mate <x>` | Search for a mate in `x` moves. |
| `movetime <x>` | Search for exactly `x` ms. |
| `infinite` | Search until `stop`; do not stop on your own, even having found a mate. |
| `perft <x>` (Stockfish extension, not in the base spec) | Move-generation node count at depth `x` — a debugging tool, not a real search. Per the SF docs' command table. |

## Engine → GUI output

| Line | Meaning |
|---|---|
| `id name <x>` / `id author <x>` | Sent once, right after `uci`. |
| `uciok` | UCI-mode handshake complete; all `option` lines have been sent. |
| `readyok` | Reply to `isready`. |
| `option name <id> type <t> [default <x>] [min <x>] [max <x>] [var <x> ...]` | Declares one configurable engine option. `t` ∈ `check` (bool) / `spin` (int, ranged) / `combo` (enum, one `var` per choice) / `button` (no value, fire-and-forget) / `string`. |
| `bestmove <move> [ponder <move>]` | Search is over. `ponder` names the move the engine expects the opponent to play, which the GUI can immediately follow with `go ponder` on. |
| `info ...` | Search progress or arbitrary text — see below. |

### `info` fields

Every `info` line is a bag of space-separated `key value...` pairs; `UciParser` should
parse it as a sequence of recognized keys rather than a fixed schema, since not every
line carries every field (see the transcripts in [fixtures/](fixtures/) — a
`movetime`-truncated line can carry `upperbound` where a normal one wouldn't; startup
lines carry only `string`).

| Field | Meaning | Source |
|---|---|---|
| `depth <x>` | Root search depth (plies) for this iteration of iterative deepening — a.k.a. `rootDepth`. | [SF terminology](https://official-stockfish.github.io/docs/stockfish-wiki/Terminology.html) |
| `seldepth <x>` | Deepest ply actually reached in this iteration's principal variation (extensions/quiescence go past `depth`). Spec requires `depth` to be present in the same line as `seldepth`. | Spec, SF terminology |
| `multipv <n>` | 1-indexed rank of this line when `MultiPV > 1`. **`multipv 1` is always the best line** — do not re-sort by score client-side (this is the exact bug KOCH-HANDOFF.md §3 records: re-ranking multipv lines by "highest White-relative score" silently picks the worst line for Black). | Spec; verified live, see `fixtures/session-sf17.1.txt` |
| `score cp <x>` / `score mate <y>` | See [§ Score perspective](#score-perspective-read-this-before-writing-the-parser) below — this is the important one. | Spec |
| `lowerbound` / `upperbound` | Present when the score is a bound, not exact — e.g. a search cut short by `movetime` before the iteration finished. Observed live: `info depth 19 ... score cp 37 upperbound ...` immediately followed by `bestmove`, in `fixtures/session-sf17.1.txt`. | Spec; verified live |
| `nodes <x>` | Total nodes searched so far. | Spec |
| `nps <x>` | Nodes per second, sent regularly. | Spec |
| `hashfull <x>` | Transposition table fill, in permille (0–1000, not 0–100). SF FAQ recommends keeping this under ~300 for full search strength. | Spec; [SF FAQ](https://official-stockfish.github.io/docs/stockfish-wiki/Stockfish-FAQ.html) |
| `tbhits <x>` | Syzygy tablebase hits so far. | Spec |
| `time <x>` | Milliseconds spent on this iteration. | Spec |
| `pv <m1> ... <mN>` | The principal variation — engine's expected best line, in UCI move notation. Always the **last** field on the line (moves can't be reliably tokenized from the middle otherwise). | Spec |
| `currmove <move>` / `currmovenumber <x>` | Root move currently being searched, and its 1-indexed position in the move-ordering list. | Spec |
| `string <text>` | Arbitrary free text, not machine-parsed. Stockfish uses this for startup diagnostics: `info string Available processors: 0-15`, `info string Using 1 thread`, `info string NNUE evaluation using nn-....nnue (...)` — one block of these precedes *every* `go`, not just the first. `UciParser` must treat `info string ...` as an opaque, ignorable line, distinct from structured `info depth ...` lines. | Verified live, both fixtures |
| `refutation <m1> <m2> ...` / `currline [cpunr] <m1> ...` / `cpuload <x>` / `sbhits <x>` | Rarely emitted by Stockfish in practice; spec-defined but not seen in the live transcripts captured here. Worth a defensive "ignore if present" case in the parser rather than active support. | Spec |

### Score perspective — read this before writing the parser

The spec's exact wording for `score cp <x>`:

> "the score from the engine's point of view in centipawns"

and for `score mate <y>`:

> "mate in y moves, not plies. If the engine is getting mated use negative values for y."

**"The engine's point of view" means relative to the side to move in the position that
was searched — not absolute, not always White.** A `score cp 120` when Black is to move
means Black is 1.2 pawns *up*, not White. This is precisely the landmine
KOCH-HANDOFF.md §3 already names: the old `updateSuggestion` code treated all scores as
White-relative and re-ranked multipv lines by "highest score," which for Black-to-move
picks the *worst* line. The fix direction implied here: `UciParser` should emit the
score exactly as received (engine/side-to-move-relative) and leave the
White-absolute conversion, if the frontend eval bar needs one, to a single explicit
`if side_to_move == Black { negate }` at the boundary — not scattered across call
sites.

## Options — live table, Stockfish 17.1

Captured directly from `echo uci | stockfish` on the binary installed at
`/usr/bin/stockfish` (full raw output: `fixtures/handshake-sf17.1.txt`). Descriptions
merged in from the
[SF docs](https://official-stockfish.github.io/docs/stockfish-wiki/UCI-Protocol-and-Stockfish-Commands.html)
where the option name alone doesn't explain intent.

| Option | Type | Default | Range | Notes |
|---|---|---|---|---|
| `Debug Log File` | string | `<empty>` | — | Path to log all engine I/O to, for debugging. |
| `NumaPolicy` | string | `auto` | `none` / `system` / `auto` / `hardware` / `custom` | Thread-to-NUMA-node binding on multi-socket machines. Not relevant to a desktop app targeting one socket. |
| `Threads` | spin | `1` | 1–1024 | CPU threads for search (Lazy SMP — see [engine-internals.md](engine-internals.md)). |
| `Hash` | spin | `16` | 1–33554432 (MB) | Transposition table size. |
| `Clear Hash` | button | — | — | Wipes the TT. Send after loading a completely unrelated position if avoiding stale-TT artifacts matters. |
| `Ponder` | check | `false` | — | Enables the ponder protocol (`go ponder` / `ponderhit`). |
| `MultiPV` | spin | `1` | 1–256 (SF docs say up to 500; the live binary caps at 256 — **doc and binary disagree, trust the binary**) | Number of best lines to report. |
| `Skill Level` | spin | `20` | 0–20 | Deliberately weakens play; 20 = full strength. |
| `Move Overhead` | spin | `10` | 0–5000 (ms) | Safety margin subtracted from available time, to absorb GUI/network latency so the engine doesn't flag on time. |
| `nodestime` | spin | `0` | 0–10000 | If nonzero, use node counts as a proxy for time instead of the wall clock — makes benchmarks/analysis deterministic across machines. |
| `UCI_Chess960` | check | `false` | — | Chess960/Fischer Random castling rules and move notation. |
| `UCI_LimitStrength` | check | `false` | — | Switches strength limiting from `Skill Level` to targeting `UCI_Elo`. |
| `UCI_Elo` | spin | `1320` | 1320–3190 | Target rating; only active when `UCI_LimitStrength` is set. Overrides `Skill Level`. |
| `UCI_ShowWDL` | check | `false` | — | Adds win/draw/loss percentages to `info` lines. |
| `SyzygyPath` | string | `<empty>` | — | Directories to search for Syzygy tablebase files. |
| `SyzygyProbeDepth` | spin | `1` | 1–100 | Minimum remaining depth before the engine bothers probing tablebases. |
| `Syzygy50MoveRule` | check | `true` | — | Whether tablebase results respect the 50-move rule. |
| `SyzygyProbeLimit` | spin | `7` | 0–7 | Max piece count to probe tablebases for. |
| `EvalFile` | string | `nn-1c0000000000.nnue` | — | Big NNUE network file (this build: 133 MiB). |
| `EvalFileSmall` | string | `nn-37f18f62d772.nnue` | — | Small NNUE network file (this build: 6 MiB), used for cheap evals in some search nodes — see [engine-internals.md](engine-internals.md). |

Non-standard developer/debugging commands, per the SF docs (not part of the UCI spec,
not needed by `Engine` for normal analysis use, listed for completeness): `bench`,
`speedtest`, `d` (ASCII board dump), `eval` (static eval, no search — explicitly *not*
recommended as a position assessment), `compiler`, `export_net`, `flip`, `perft`,
`help`/`license`.

## Implications for `UciProcess` / `UciParser` / `Engine`

- `UciParser` needs at least three line shapes, not one grammar: `id`/`option`/`uciok`
  (handshake, parsed once), `bestmove` (terminal), and `info` (either `info string ...`
  free text, or structured `info depth ...` key/value pairs, order not guaranteed
  fixed — parse by keyword, not position).
- The pre-`uci` banner line and the per-`go` `info string` diagnostic block are both
  real, both unprompted, and both need an explicit "ignore, don't error" path — they
  aren't edge cases, they appear on *every* handshake and *every* search respectively.
- `multipv` lines for one iteration arrive **interleaved with increasing `multipv`
  index at the same `depth`** before moving to the next `depth` (see
  `fixtures/session-sf17.1.txt`, third `go`) — a UI showing "current best lines" should
  key its rows by `multipv` index and update in place, not append.
- The `Score perspective` fix belongs in `Engine` or above, not in `UciParser` —
  `UciParser` should stay a faithful `&str → InfoLine` mapping with no side-to-move
  knowledge, per the handoff's "pure, no I/O, no state" design goal for it.
