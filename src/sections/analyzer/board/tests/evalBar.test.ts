import { describe, expect, it } from "vitest";
import {
  EVAL_UNKNOWN_LABEL,
  evalBarWhiteFraction,
  formatEvalScore,
  topLineMove,
  topLineScore,
} from "../evalBar";
import type { EvalScore } from "../../types";

const cp = (centipawns: number): EvalScore => ({ kind: "cp", centipawns });
const mate = (movesToMate: number): EvalScore => ({
  kind: "mate",
  movesToMate,
});

describe("evalBarWhiteFraction", () => {
  it("sits level for a drawn position and for no score at all", () => {
    expect(evalBarWhiteFraction(cp(0))).toBe(0.5);
    expect(evalBarWhiteFraction(null)).toBe(0.5);
  });

  it("moves toward White for a positive score and Black for a negative one", () => {
    expect(evalBarWhiteFraction(cp(250))).toBe(0.75);
    expect(evalBarWhiteFraction(cp(-250))).toBe(0.25);
  });

  it("clamps rather than overflowing past either end", () => {
    expect(evalBarWhiteFraction(cp(9000))).toBe(1);
    expect(evalBarWhiteFraction(cp(-9000))).toBe(0);
  });

  it("pins fully for a mate, whoever is to move", () => {
    expect(evalBarWhiteFraction(mate(3))).toBe(1);
    expect(evalBarWhiteFraction(mate(-2))).toBe(0);
  });

  // The §3 landmine, as a test: a score is White-relative regardless of
  // whose turn it is, so the same number must give the same bar. Nothing
  // in this module takes a side-to-move argument that could flip it.
  it("reads a Black-favouring score the same way in every position", () => {
    expect(evalBarWhiteFraction(cp(-150))).toBeLessThan(0.5);
  });
});

describe("formatEvalScore", () => {
  it("renders centipawns as signed pawns", () => {
    expect(formatEvalScore(cp(125))).toBe("+1.25");
    expect(formatEvalScore(cp(-30))).toBe("-0.30");
  });

  it("drops the sign at dead even", () => {
    expect(formatEvalScore(cp(0))).toBe("0.00");
  });

  it("renders mates with their side's sign", () => {
    expect(formatEvalScore(mate(3))).toBe("+M3");
    expect(formatEvalScore(mate(-2))).toBe("-M2");
  });

  it("says nothing rather than zero when there is no score", () => {
    expect(formatEvalScore(null)).toBe(EVAL_UNKNOWN_LABEL);
  });
});

describe("topLineScore", () => {
  it("takes the first line, not the highest-scoring one", () => {
    // Black to move: the best line is the *least* positive, because every
    // score is White-relative. Picking the maximum here is exactly the old
    // `updateSuggestion` bug, which pointed the arrow at a blunder.
    const lines = [{ score: cp(-180) }, { score: cp(-40) }, { score: cp(20) }];

    expect(topLineScore(lines)).toEqual(cp(-180));
  });

  it("has no score for an empty or missing line list", () => {
    expect(topLineScore([])).toBeNull();
    expect(topLineScore(undefined)).toBeNull();
  });
});

describe("topLineMove", () => {
  it("takes line 1's first move, not the best-scoring line's", () => {
    // Black to move, every score White-relative: line 1 is the most
    // negative. The highest-scoring line here is Black's *worst* option.
    const lines = [
      { score: cp(-180), moves: ["c6a5", "b3c2"] },
      { score: cp(20), moves: ["h7h6"] },
    ];
    expect(topLineMove(lines)).toBe("c6a5");
  });

  it("has no move before the engine reports one", () => {
    expect(topLineMove([])).toBeNull();
    expect(topLineMove(undefined)).toBeNull();
  });
});
