import { describe, expect, it } from "vitest";
import type { GameStateView } from "../../../api/bindings/GameStateView";
import { newerState } from "../GameContext";

const stateAfter = (moves: string[]): GameStateView => ({
  turn: moves.length % 2 === 0 ? "white" : "black",
  pieces: [],
  legal_moves: {},
  move_history: moves,
  last_move: null,
  opening_name: null,
  result: "Unfinished",
  human_score: null,
  white_remaining_ms: 600_000,
  black_remaining_ms: 600_000,
  elapsed_this_turn_ms: 0,
});

describe("newerState", () => {
  it("keeps the engine's reply when the response to the move it answers arrives after it", () => {
    const reply = stateAfter(["Qe4+", "Kg5"]);

    expect(newerState(reply, stateAfter(["Qe4+"]))).toBe(reply);
  });

  it("takes a state with more moves", () => {
    const reply = stateAfter(["e4", "e5"]);

    expect(newerState(stateAfter(["e4"]), reply)).toBe(reply);
  });

  it("takes a state with the same moves, so a game's end replaces the position it ended in", () => {
    const resigned: GameStateView = {
      ...stateAfter(["e4"]),
      result: "WhiteWin",
    };

    expect(newerState(stateAfter(["e4"]), resigned)).toBe(resigned);
  });
});
