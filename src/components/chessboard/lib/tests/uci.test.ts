import { describe, expect, it } from "vitest";
import { squareName } from "../squareName";
import { parseSquare, parseUciMove } from "../uci";

describe("parseSquare", () => {
  // Rank 0 is the eighth rank, so the corners are what catch a flipped
  // digit — the same inversion squareName does in the other direction.
  it("parses the corners", () => {
    expect(parseSquare("a8")).toEqual({ rank: 0, file: 0 });
    expect(parseSquare("h8")).toEqual({ rank: 0, file: 7 });
    expect(parseSquare("a1")).toEqual({ rank: 7, file: 0 });
    expect(parseSquare("h1")).toEqual({ rank: 7, file: 7 });
  });

  it("round-trips through squareName", () => {
    for (const name of ["a1", "e4", "d5", "h8", "c6"]) {
      const square = parseSquare(name);
      expect(square).not.toBeNull();
      expect(squareName(square!)).toBe(name);
    }
  });

  it("rejects anything that isn't a square", () => {
    expect(parseSquare("i4")).toBeNull();
    expect(parseSquare("e9")).toBeNull();
    expect(parseSquare("e0")).toBeNull();
    expect(parseSquare("e")).toBeNull();
    expect(parseSquare("")).toBeNull();
  });
});

describe("parseUciMove", () => {
  it("splits a move into its two squares", () => {
    expect(parseUciMove("e2e4")).toEqual({
      from: { rank: 6, file: 4 },
      to: { rank: 4, file: 4 },
    });
  });

  it("ignores the promotion piece", () => {
    expect(parseUciMove("e7e8q")).toEqual({
      from: { rank: 1, file: 4 },
      to: { rank: 0, file: 4 },
    });
  });

  it("returns null rather than half a move", () => {
    expect(parseUciMove("e2")).toBeNull();
    expect(parseUciMove("z2e4")).toBeNull();
    expect(parseUciMove("")).toBeNull();
  });
});
