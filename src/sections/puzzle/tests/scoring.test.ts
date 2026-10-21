import { describe, expect, it } from "vitest";
import {
  MAX_STREAK_MULTIPLIER,
  puzzlePoints,
  streakMultiplier,
} from "../scoring";

describe("puzzlePoints", () => {
  it("is the rating in hundreds for a clean first solve", () => {
    expect(puzzlePoints(1500, 0, 0)).toBe(15);
  });

  it("halves for every mistake, rounding down", () => {
    expect(puzzlePoints(1500, 1, 0)).toBe(7);
    expect(puzzlePoints(1500, 2, 0)).toBe(3);
  });

  it("is worth more the longer the clean run it extends", () => {
    // 15 × 1.3
    expect(puzzlePoints(1500, 0, 3)).toBe(20);
  });

  it("stops growing once the multiplier is capped", () => {
    expect(puzzlePoints(1500, 0, 25)).toBe(15 * MAX_STREAK_MULTIPLIER);
  });
});

describe("streakMultiplier", () => {
  it("adds nothing before there's a run", () => {
    expect(streakMultiplier(0)).toBe(1);
  });

  it("reaches its cap ten puzzles in, and stays there", () => {
    expect(streakMultiplier(10)).toBe(MAX_STREAK_MULTIPLIER);
    expect(streakMultiplier(25)).toBe(MAX_STREAK_MULTIPLIER);
  });
});
