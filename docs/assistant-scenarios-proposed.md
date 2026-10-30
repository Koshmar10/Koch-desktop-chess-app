# Assistant scenarios — proposed

Five scenarios a capable coach should answer reliably, and the tools they imply.
**Proposed, not agreed.** The working list is [assistant-scenarios.md](assistant-scenarios.md);
reconcile the two before building tools.

## Scenarios (original wording)

- Cum este pozitia in care sunt?
- Ce optiuni am in pozitia asta si de ce ar trebui sa aleg ce imi zici tu?
- Unde a inceput meciul sa mearga prost pentru mine (point of no return)?
- Am gresit opening-ul? (am respectat ideile principale? am respectat seria de mutari?)
- Exista tipare pe care le repet?

## Principle

The LLM explains; tools compute. The model never works out evals, turning points or
pattern counts itself.

## Scope and tools

| # | Scenario | Scope | Tool | Returns |
|---|---|---|---|---|
| 1 | How is my position? | position | `analyse_position(ply \| fen)` | eval from the user's side, best move, material, structure and king-safety findings |
| 2 | Options and why | position | `candidate_lines(fen, n)` | top n multi-PV lines with eval and first moves; doubles as the legality check |
| 3 | Point of no return | game | `find_turning_point(game_id)` | ply, eval before/after, played vs best move |
| 4 | Opening check | game | `opening_report(game_id)` | opening name, ply where the game left book, book move there |
| 5 | Repeated patterns | many games | `query_mistakes(filter)`, `player_stats(group_by)` | counts plus example game/ply references |

## Design notes

- **Point of no return** is defined in code, e.g. the last ply after which the eval never
  returns within ±1.0 of equal. It is not the same as the largest single blunder; the
  tool can return both.
- **Opening ideas** (plans, structures) come from the model's own knowledge and must be
  flagged as general knowledge, not tool-verified. This is the highest hallucination risk.
- **Scenario 5 needs mistake tags** (missed fork, hung piece, back-rank weakness), i.e.
  detectors in the analyzer. Treat it as its own epic.
- **Perspective:** every tool returns evals from the user's side, never White-relative,
  so the sign-convention landmine in KOCH-HANDOFF.md §3 stays out of the prompt.
- **Small results:** summaries and a few top lines, not full PVs or whole games; every
  token returns as input on the next tool round.
- **Cross-game access:** scenario 5 breaks the one-chat-per-game assumption. Tools must be
  able to read other games, or there needs to be a non-game chat.
- **UI state:** the frontend sends the selected ply/FEN with each message, so "the
  position I'm in" is not guessed.

## Build order

1 and 2 first (everything reuses them), then 3, then 4. Scenario 5 later.
