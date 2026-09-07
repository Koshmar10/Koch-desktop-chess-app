# Stockfish investigation

Research for [KOCH-HANDOFF.md §5](../../KOCH-HANDOFF.md) / milestone M4: owning the UCI
communication layer instead of depending on the `stockfish` crate. This folder is
findings, not a design doc — it documents what Stockfish actually does and says, with
sources, so the `UciProcess` / `UciParser` / `Engine` implementation (KOCH-8) can be
written against verified facts instead of half-remembered protocol trivia.

## Method

Two kinds of source, cross-checked against each other:

1. **The locally installed binary** — `/usr/bin/stockfish`, **Stockfish 17.1**, built
   with `g++ 15.1.1`, `x86-64-bmi2` (`AVX2 BMI2 SSE41 SSSE3 SSE2 POPCNT`). Every option
   name/type/default/range in [uci-protocol.md](uci-protocol.md) is copy-pasted from
   this binary's own `uci` response, not transcribed from docs that may lag the running
   version. Raw transcripts are kept in [fixtures/](fixtures/) — they can seed
   `UciParser` test fixtures directly later, per the handoff's plan to "record real
   Stockfish output into fixture files once."
2. **Written documentation** — the official Stockfish docs site, the canonical UCI
   protocol spec text, and chessprogramming.org — fetched 2026-08-22. Every claim in
   these files that comes from a doc, not from running the binary, carries an inline
   link.

Where the two disagreed or a doc was thin, that's noted rather than papered over.

## Files

- **[uci-protocol.md](uci-protocol.md)** — the wire protocol: command lifecycle,
  every GUI→engine and engine→GUI command, the full live option table, and the `info`
  line fields. Read this first — it's what `UciParser` parses and `Engine` drives.
- **[engine-internals.md](engine-internals.md)** — what happens between `go` and
  `bestmove`: iterative deepening, pruning/reductions/extensions, NNUE evaluation,
  the transposition table, Lazy SMP threading, Syzygy tablebases. Background for
  understanding *why* the output looks the way it does (e.g. why `nodes` jumps
  non-monotonically with depth, why a shallow search can already look confident).
- **[licensing-and-distribution.md](licensing-and-distribution.md)** — GPLv3
  obligations, and why talking to Stockfish over stdin/stdout (which Koch already does)
  keeps the rest of the app clear of them. Relevant to the M4 "bundled vs prerequisite"
  decision the handoff leaves open.
- **[fixtures/](fixtures/)** — raw, unedited transcripts from the local Stockfish 17.1
  binary: a bare handshake, and a longer session covering single-PV search, a
  `movetime`-bounded search, and a 3-line `MultiPV` search.

## Headline finding: the score-perspective landmine, confirmed at the spec level

KOCH-HANDOFF.md §3 already flags a sign-convention bug from the old codebase. The root
cause is now pinned to the exact spec sentence: `score cp <x>` is defined as **"the
score from the engine's point of view in centipawns"** — i.e. relative to the side to
move, never absolute-White — per the
[canonical UCI protocol spec](https://gist.github.com/DOBRO/2592c6dad754ba67e6dcaec8c90165bf).
Full detail and the implication for `UciParser`/`Engine` is in
[uci-protocol.md § Score perspective](uci-protocol.md#score-perspective-read-this-before-writing-the-parser).
