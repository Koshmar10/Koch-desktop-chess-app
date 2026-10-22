# Assistant scenarios

What the Koch assistant has to handle, what data answers each kind of
question, and where that data comes from. This table decides three things:

- what goes into the **context** sent with every request (KOCH-19),
- which **tools** the model can call (KOCH-20, KOCH-28),
- the **golden questions** the assistant is evaluated against (KOCH-27).

**Context** means it's sent with every question. Use it for what almost
every question needs, since a tool round costs a second or more and re-sends
the whole conversation. **Tool** means the model asks for it when it needs
it.

## Scenarios

| # | Scenario | Example questions | Data needed | Source | Status |
|---|---|---|---|---|---|
| S1 | Position | "What's the plan here?" · "What are the threats?" · "Is my king safe?" | FEN, eval, side to move, `PositionFindings` (hanging pieces, pins, forks, king safety, pawn structure), which side the user played | context | not yet |
| S2 | Last move | "Why was this a mistake?" · "What should I have played?" | eval before and after, move quality, best move at the previous ply | context + engine | not yet |
| S3 | Another ply | "What about move 12?" · "Why did the eval drop on move 20?" | that ply's position and analysis | tool: `get_position(ply)` | not yet |
| S4 | Whole game | "Where did I go wrong?" · "What was the turning point?" | mistakes and blunders with their eval swings, per-side accuracy | tool: `get_game_review` | not yet |
| S5 | What if | "What happens after Bxh7+?" · "Is Nd5 playable?" | play a line from the position, then evaluate it | tool: `explore_line(moves)` | not yet |
| S6 | Player | "What are my weaknesses?" · "Am I improving?" · "What should I train?" | rating trend, accuracy, results by opening, blunder rate by phase | rating in context; profile via tool `get_player_profile` | not yet |
| S7 | Opening | "What opening is this?" · "Where did I leave theory?" | opening name and ECO for the position | tool: `lookup_opening` | not yet |
| S8 | Concept | "What's a minority attack?" · "When should I trade queens?" | the model's own knowledge | nothing | works now |
| S9 | Follow-up | "Why?" · "And if Black doesn't take?" | the earlier turns | backend conversation | works now |

## Edge cases

Each of these needs a defined answer, not whatever the model improvises.

| Case | What the assistant should do |
|---|---|
| Sandbox (no `game_id`) | No stored analysis or game review. Findings come from the board; the engine tool still works. |
| Game not analysed yet | Say the evals aren't there yet and suggest running the analysis. The engine tool still works. |
| Start position (ply −1) | There's no last move. S2 questions get "no move has been played yet". |
| No API key | Refused before any request, with the message pointing at Settings. *(done)* |
| No text in the reply (a refusal, say) | Shown as an error: "The model sent no text reply." *(done)* |
| Question not about chess | A short answer at most, then back to the game. |
| Text from an imported PGN (headers, comments) | Treated as data. Instructions found inside it are not followed. |
| A second question while one is pending | Refused by the backend until the first is answered. *(done)* |

## Golden questions

Real questions from your own games, for KOCH-27. Each one says what a
correct answer must contain and what it must not claim. Grade by hand
against these two columns, per config (model, effort, prompt, tool set).

| ID | Game · ply | Scenario | Question | Must contain | Must not claim |
|---|---|---|---|---|---|
| G1 | *game id · ply* | S2 | *e.g. "Why was this a mistake?"* | *e.g. the piece left hanging and the move that wins it* | *e.g. a move that isn't legal in the position* |
| G2 | | | | | |
| G3 | | | | | |

Aim for 15–20, with at least two per scenario and one per edge case that
the model (not the code) handles.

## What to log per turn

While profiling, record for every answered question:

- the question and the scenario it belongs to,
- each tool called, with its arguments,
- the number of API rounds,
- the token counts (`TurnUsage`) and the time to answer.

That log shows what the assistant actually does with each scenario, and
whether a cheaper model or a lower effort level is good enough.
