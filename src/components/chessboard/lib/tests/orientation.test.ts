import { describe, expect, it } from "vitest";
import { fromScreen, toScreen } from "../orientation";

describe("toScreen", () => {
  it("draws a square where it is when not flipped", () => {
    expect(toScreen({ rank: 0, file: 0 }, false)).toEqual([0, 0]);
    expect(toScreen({ rank: 3, file: 5 }, false)).toEqual([3, 5]);
  });

  it("mirrors both axes when flipped", () => {
    expect(toScreen({ rank: 0, file: 0 }, true)).toEqual([7, 7]);
    expect(toScreen({ rank: 7, file: 7 }, true)).toEqual([0, 0]);
    expect(toScreen({ rank: 2, file: 3 }, true)).toEqual([5, 4]);
  });
});

describe("fromScreen", () => {
  it("reads a cell as the square drawn there when not flipped", () => {
    expect(fromScreen([3, 5], false)).toEqual({ rank: 3, file: 5 });
  });

  it("mirrors both axes when flipped", () => {
    expect(fromScreen([0, 0], true)).toEqual({ rank: 7, file: 7 });
    expect(fromScreen([5, 4], true)).toEqual({ rank: 2, file: 3 });
  });

  // Rendering and hit-testing have to agree, or a click lands on a
  // different square from the one drawn under the pointer.
  it("undoes toScreen in either orientation", () => {
    const square = { rank: 2, file: 5 };

    expect(fromScreen(toScreen(square, true), true)).toEqual(square);
    expect(fromScreen(toScreen(square, false), false)).toEqual(square);
  });
});
