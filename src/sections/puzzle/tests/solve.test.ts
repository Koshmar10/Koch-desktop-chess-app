import { describe, expect, it } from "vitest";
import type { ChessPuzzle } from "../../../api/bindings/ChessPuzzle";
import { solveFor, startSolve } from "../solve";

const puzzle = (id: string): ChessPuzzle => ({
  id,
  rating: 1500,
  themes: [],
  positions: [],
  start: 42,
  legal_moves: [],
  turn: "black",
});

describe("startSolve", () => {
  it("starts one position early, on the opponent's move", () => {
    expect(startSolve(puzzle("a"))).toEqual({
      puzzleId: "a",
      currentPly: 41,
      turn: "white",
      mistakes: [],
      earnedPoints: 0,
    });
  });
});

describe("solveFor", () => {
  it("keeps the progress stored for the same puzzle", () => {
    const stored = { ...startSolve(puzzle("a")), currentPly: 44 };

    expect(solveFor(stored, puzzle("a"))).toBe(stored);
  });

  it("starts fresh when what's stored is another puzzle's", () => {
    const stored = { ...startSolve(puzzle("a")), currentPly: 44 };

    expect(solveFor(stored, puzzle("b"))).toEqual(startSolve(puzzle("b")));
  });

  it("starts fresh when nothing is stored yet", () => {
    expect(solveFor(null, puzzle("a"))).toEqual(startSolve(puzzle("a")));
  });
});
