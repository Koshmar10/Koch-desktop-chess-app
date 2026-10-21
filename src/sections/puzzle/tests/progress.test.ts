import { describe, expect, it } from "vitest";
import type { ChessPuzzle } from "../../../api/bindings/ChessPuzzle";
import { newProgress, progressFor } from "../progress";

const puzzle = (id: string): ChessPuzzle => ({
  id,
  rating: 1500,
  themes: [],
  positions: [],
  start_ply: 42,
  legal_moves: [],
  solver_color: "black",
});

describe("newProgress", () => {
  it("starts one position early, on the opponent's move", () => {
    expect(newProgress(puzzle("a"))).toEqual({
      puzzleId: "a",
      currentPly: 41,
      viewedPly: 41,
      turn: "white",
      mistakes: [],
      earnedPoints: 0,
      solvedCard: "hidden",
    });
  });
});

describe("progressFor", () => {
  it("keeps the progress stored for the same puzzle", () => {
    const stored = { ...newProgress(puzzle("a")), currentPly: 44 };

    expect(progressFor(stored, puzzle("a"))).toBe(stored);
  });

  it("starts fresh when what's stored is another puzzle's", () => {
    const stored = { ...newProgress(puzzle("a")), currentPly: 44 };

    expect(progressFor(stored, puzzle("b"))).toEqual(newProgress(puzzle("b")));
  });

  it("starts fresh when nothing is stored yet", () => {
    expect(progressFor(null, puzzle("a"))).toEqual(newProgress(puzzle("a")));
  });
});
