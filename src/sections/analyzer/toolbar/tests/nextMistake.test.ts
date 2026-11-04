import { describe, expect, it } from "vitest";
import type { MoveQuality } from "../../../../api/bindings/MoveQuality";
import { START_POSITION_PLY } from "../../../../lib/ply";
import { nextMistakePly, previousMistakePly } from "../nextMistake";

// White's moves on even plies, Black's on odd ones.
const QUALITIES: (MoveQuality | null)[] = [
  "Good", // 0 white
  "Inaccuracy", // 1 black
  "Excellent", // 2 white
  "Blunder", // 3 black
  "Mistake", // 4 white
  null, // 5 black, ungraded
];

describe("nextMistakePly", () => {
  it("finds the first mistake or blunder after the viewed ply", () => {
    expect(nextMistakePly(QUALITIES, START_POSITION_PLY, null)).toBe(3);
  });

  it("doesn't count an inaccuracy as a mistake", () => {
    expect(
      nextMistakePly(["Inaccuracy", "Good"], START_POSITION_PLY, null),
    ).toBe(null);
  });

  it("skips the move on screen itself", () => {
    expect(nextMistakePly(QUALITIES, 3, null)).toBe(4);
  });

  it("only counts the given side's moves", () => {
    expect(nextMistakePly(QUALITIES, START_POSITION_PLY, "white")).toBe(4);
  });

  it("is null once there are none left", () => {
    expect(nextMistakePly(QUALITIES, 4, null)).toBe(null);
  });
});

describe("previousMistakePly", () => {
  it("finds the last mistake or blunder before the viewed ply", () => {
    expect(previousMistakePly(QUALITIES, 5, null)).toBe(4);
  });

  it("skips the move on screen itself", () => {
    expect(previousMistakePly(QUALITIES, 4, null)).toBe(3);
  });

  it("only counts the given side's moves", () => {
    expect(previousMistakePly(QUALITIES, 5, "black")).toBe(3);
  });

  it("is null before the first one", () => {
    expect(previousMistakePly(QUALITIES, 3, null)).toBe(null);
    expect(previousMistakePly(QUALITIES, START_POSITION_PLY, null)).toBe(null);
  });
});
