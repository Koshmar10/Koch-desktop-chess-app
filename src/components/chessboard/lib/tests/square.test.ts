import { describe, expect, it } from "vitest";
import { isDarkSquare, sameSquare } from "../square";

describe("sameSquare", () => {
  it("is true for different objects naming the same square", () => {
    expect(sameSquare({ rank: 3, file: 5 }, { rank: 3, file: 5 })).toBe(true);
  });

  it("is false for squares that differ in rank or file", () => {
    expect(sameSquare({ rank: 3, file: 5 }, { rank: 3, file: 6 })).toBe(false);
    expect(sameSquare({ rank: 3, file: 5 }, { rank: 4, file: 5 })).toBe(false);
  });

  it("is false whenever either side is null", () => {
    expect(sameSquare(null, { rank: 3, file: 5 })).toBe(false);
    expect(sameSquare({ rank: 3, file: 5 }, null)).toBe(false);
    expect(sameSquare(null, null)).toBe(false);
  });
});

describe("isDarkSquare", () => {
  // a8 is light and h1 is light — the corners pin the parity.
  it("colours the corners the way a real board does", () => {
    expect(isDarkSquare({ rank: 0, file: 0 })).toBe(false); // a8
    expect(isDarkSquare({ rank: 7, file: 7 })).toBe(false); // h1
    expect(isDarkSquare({ rank: 7, file: 0 })).toBe(true); // a1
    expect(isDarkSquare({ rank: 0, file: 7 })).toBe(true); // h8
  });
});
