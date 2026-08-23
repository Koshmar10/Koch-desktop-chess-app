# Licensing — relevant to the M4 "bundled vs prerequisite" decision

KOCH-HANDOFF.md's M4 entry leaves open whether the Stockfish binary is bundled with
Koch or expected as a system prerequisite, with discovery replacing the old hardcoded
`/usr/bin/stockfish` path. This is the licensing context for that decision.

Source: [Stockfish Developers docs](https://official-stockfish.github.io/docs/stockfish-wiki/Developers.html).

## The core facts

- Stockfish is **GNU GPLv3**.
- Distributing Stockfish (bundling the binary with Koch) requires including **the
  license and the full source code**, or a reference to where it can be obtained.
- Any **modifications** to Stockfish itself must also be released under GPLv3.
- The license permits distribution, sale, and integration with proprietary software
  **provided Stockfish and the application "communicate at arm's length"** rather than
  being a single combined/linked program.

## Why this doesn't force Koch's own code under GPL

Koch already talks to Stockfish as a **separate subprocess over stdin/stdout** — never
linking against Stockfish's source or object code. That's precisely the "arm's length"
communication the license describes as staying outside the combined-work / copyleft
boundary. Nothing about moving from the `stockfish` crate to an in-house `UciProcess`
changes this — it's still process-boundary communication either way, just without a
third-party crate mediating it.

## What bundling *would* still require, if chosen

If the Stockfish binary ships inside Koch's installer/bundle (as opposed to Koch
finding a system-installed `stockfish` at runtime), Koch would need to:

- Include Stockfish's `LICENSE`/`COPYING` file (or link to it) alongside the bundle.
- Make Stockfish's source available or link to it — trivially satisfied by pointing at
  the [official Stockfish repo](https://github.com/official-stockfish/Stockfish), since
  Koch wouldn't be shipping a modified build.

Neither obligation touches Koch's own source — they're about the Stockfish binary
specifically. This makes bundling a packaging-size/UX tradeoff (skip the "install
Stockfish yourself" step vs. a larger installer and per-platform binary management), not
a licensing one.
