import { describe, expect, it } from "vitest";
import { squareName } from "../squareName";

describe("squareName", () => {
  // The inversion is the whole point of the function: rank 0 is the eighth
  // rank, not the first, so the corners are what catch a sign slip.
  it("names the corners", () => {
    expect(squareName({ rank: 0, file: 0 })).toBe("a8");
    expect(squareName({ rank: 0, file: 7 })).toBe("h8");
    expect(squareName({ rank: 7, file: 0 })).toBe("a1");
    expect(squareName({ rank: 7, file: 7 })).toBe("h1");
  });

  it("names the squares the starting pieces stand on", () => {
    // White's king starts at rank 7, file 4 — e1, not e8.
    expect(squareName({ rank: 7, file: 4 })).toBe("e1");
    expect(squareName({ rank: 0, file: 4 })).toBe("e8");
    // And a white pawn's opening double-step lands on e4.
    expect(squareName({ rank: 4, file: 4 })).toBe("e4");
  });
});
