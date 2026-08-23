# What happens inside Stockfish, between `go` and `bestmove`

Background for reading `info` output with correct intuition — not needed to *implement*
`UciProcess`/`UciParser` (that only needs [uci-protocol.md](uci-protocol.md)), but
useful for `Engine`'s callers and for explaining odd-looking output (e.g. `nodes`
jumping unevenly between depths, or a `depth 4` line suddenly reporting a much better
score than `depth 3`).

Sources: [Stockfish FAQ](https://official-stockfish.github.io/docs/stockfish-wiki/Stockfish-FAQ.html),
[Stockfish terminology glossary](https://official-stockfish.github.io/docs/stockfish-wiki/Terminology.html),
[Stockfish Advanced Topics — Syzygy](https://official-stockfish.github.io/docs/stockfish-wiki/Advanced-topics.html),
[chessprogramming.org — Lazy SMP](https://www.chessprogramming.org/Lazy_SMP),
[Stockfish blog — Introducing NNUE Evaluation](https://stockfishchess.org/blog/2020/introducing-nnue-evaluation/),
and the live `compiler`/`uci` output of the local Stockfish 17.1 binary.

## The search loop: iterative deepening + PVS, not plain minimax

Stockfish does **not** do a uniform-depth brute-force search. Per the SF FAQ, "depth"
counts an outer **iterative deepening** loop — search to depth 1, then 2, then 3, reusing
information (transposition table entries, move ordering, killer moves) from each
shallower pass to make the next one faster and better-ordered. `info depth <x>` is this
loop's counter, i.e. `rootDepth`, not a promise that every branch was searched to
exactly that depth.

Within one iteration, the tree is heavily *non-uniform*, per the FAQ:

- **Pruning** cuts branches that look unpromising without searching them at all —
  e.g. **null-move pruning** (let the opponent move twice; if you're still fine, this
  branch is probably safe to skip) and **futility pruning** (a quiet move unlikely to
  change the evaluation enough to matter near the search frontier).
- **Reductions** (**late move reductions**, **late move pruning**) search
  lower-priority branches — later in the move-ordering list, so already guessed to be
  worse — at reduced depth, or skip them outright past the first few.
- **Extensions** search *important* lines deeper than the nominal depth — checks,
  forcing captures — because cutting those off early is where tactical blunders come
  from.

Net effect, quoted from the FAQ: *"an engine's tree is not uniformly cut off at a single
depth; most lines end earlier, some go deeper."* This is why `seldepth` (deepest ply
actually reached) is routinely well past `depth`, and why score swings between
consecutive `depth` lines aren't noise — a reduction/extension boundary shifting by one
ply can flip which line looks best.

At the leaves, a **quiescence search** keeps expanding captures/checks past the nominal
depth before calling the static evaluator, specifically to avoid evaluating a position
mid-capture-sequence (the classic "horizon effect").

Move ordering — trying the most promising move first at each node — is what makes
alpha-beta pruning (here, its refinement **Principal Variation Search**) effective at
all; a badly-ordered search degrades toward the unpruned worst case.

## Evaluation: NNUE

Since the classical hand-crafted evaluation (**HCE**) was removed in August 2023,
Stockfish scores every leaf with **NNUE** — "Efficiently Updatable Neural Network."
Per the [Stockfish blog post introducing it](https://stockfishchess.org/blog/2020/introducing-nnue-evaluation/)
and chessprogramming.org, NNUE originated in Shogi engines (YaneuraOu, 2018) and was
ported to chess/Stockfish in 2019. The "efficiently updatable" part is the point: after
a normal chess move, only a small slice of the network's input needs recomputing rather
than the whole forward pass, which is what makes a neural eval cheap enough to call at
every leaf of a fast alpha-beta search — unlike, say, a full policy/value net evaluated
once per MCTS node the way AlphaZero-style engines do it.

The live binary loads **two** network files, confirmed from its own `uci` output:

```
option name EvalFile type string default nn-1c0000000000.nnue
option name EvalFileSmall type string default nn-37f18f62d772.nnue
```

and reports, per search, which it used and at what size:

```
info string NNUE evaluation using nn-1c0000000000.nnue (133MiB, (22528, 3072, 15, 32, 1))
info string NNUE evaluation using nn-37f18f62d772.nnue (6MiB, (22528, 128, 15, 32, 1))
```

(architecture tuple = layer sizes; not decoded further here, out of scope for the UCI
layer). Recent Stockfish versions use the small net for cheap/simple positions and the
large net where the position is complex enough to warrant it — both are always loaded,
selection happens per-position inside the engine, invisible at the UCI level beyond
these two startup `info string` lines.

## The transposition table (`Hash`)

A hash table keyed by position, storing prior search results (best move, score, depth,
bound type) so transpositions — the same position reached by a different move order —
are recognized instead of re-searched. `hashfull` in `info` lines reports how full it
is, in permille. The FAQ's practical guidance: keep average `hashfull` under ~30% during
a search for full search quality — a table thrashing near 100% full starts evicting
useful entries mid-search.

## Threading: Lazy SMP, not tree-splitting

Stockfish 7 (Jan 2016) switched to **Lazy SMP** and has used it since
([chessprogramming.org](https://www.chessprogramming.org/Lazy_SMP)). The mental model
that matters for `Engine`: setting `Threads > 1` does **not** partition the search tree
across threads. Instead, every thread searches the *same* root position independently,
sharing one transposition table; threads are seeded with slightly different depths
and/or move-ordering so they don't do fully duplicate work, and the shared TT lets a
faster/luckier thread's discoveries speed up the others. The final move is chosen by
a form of depth/score-weighted voting across threads, not by "thread 0 wins."

Practically: from the UCI layer's point of view, `Threads` is a single opaque knob —
Stockfish still speaks UCI as one process, one `info` stream, one `bestmove`. Nothing
about the wire protocol changes with thread count; only search quality/speed does. The
SF FAQ recommends leaving 1–2 threads free of the total core count for OS/GUI
responsiveness.

## Syzygy tablebases (optional, `SyzygyPath`)

Endgame databases keyed by exact material (up to 7 pieces, ~17TB for the full 7-piece
set per the [Advanced Topics doc](https://official-stockfish.github.io/docs/stockfish-wiki/Advanced-topics.html)).
Two distinct behaviors worth knowing before surfacing `tbhits` in a UI:

- **Position is already in the tablebase** (≤ `SyzygyProbeLimit` pieces on the board):
  the engine restricts its root move choice to the set of tablebase-optimal moves
  (preserving the win/draw under the 50-move rule), then searches *within* that set to
  additionally prefer the shortest mate. Per the doc, verbatim-adjacent: it will not
  move instantly just because the outcome is known, unless only one such move exists.
- **Position is not yet in the tablebase, but search reaches one**: `tbhits` increments
  as tablebase positions are probed inside the search tree; a sudden very large score
  (approaching Stockfish's mate-encoding range, "near 200.00" per the doc's example)
  signals the search found a forced path into a won tablebase position, not necessarily
  an immediate mate on the board.

Not relevant to Koch until tablebase support is explicitly scoped — noted here so a
future "why did `tbhits` jump and the eval bar pin to the ceiling" question isn't a
surprise.

## Terminology quick-reference

Definitions below are Stockfish's own, from the
[terminology glossary](https://official-stockfish.github.io/docs/stockfish-wiki/Terminology.html),
condensed:

| Term | Meaning |
|---|---|
| Ply | One half-move (one side's move). |
| Depth / rootDepth | Iterative-deepening loop counter at the root. |
| Seldepth | Deepest ply actually reached in the current PV. |
| PV (principal variation) | The move sequence the engine currently considers best. |
| MultiPV | Reporting the top *N* independent PVs instead of just one. |
| TT (transposition table) | Cache of previously-searched positions' results. |
| Lazy SMP | Multi-threaded search: independent per-thread searches sharing one TT. |
| Null move | A (illegal) "pass," used inside null-move pruning, not a legal option ever sent to the GUI. |
| NMP / LMP / LMR | Null-move / late-move pruning / late-move reductions — see § above. |
| Quiescence search | Extension past nominal depth through captures/checks before evaluating a leaf. |
| HCE | Hand-crafted evaluation — removed from Stockfish in Aug 2023, replaced entirely by NNUE. |
| NNUE | Efficiently Updatable Neural Network — current evaluation method. |
