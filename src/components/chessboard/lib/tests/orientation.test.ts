import { describe, expect, it } from "vitest";
import { flipCoords } from "../orientation";

describe("flipCoords", () => {
  it("is the identity when not flipped", () => {
    expect(flipCoords(0, 0, false)).toEqual([0, 0]);
    expect(flipCoords(3, 5, false)).toEqual([3, 5]);
  });

  it("mirrors both axes when flipped", () => {
    expect(flipCoords(0, 0, true)).toEqual([7, 7]);
    expect(flipCoords(7, 7, true)).toEqual([0, 0]);
    expect(flipCoords(2, 3, true)).toEqual([5, 4]);
  });

  it("is its own inverse", () => {
    const once = flipCoords(2, 5, true);
    const twice = flipCoords(once[0], once[1], true);

    expect(twice).toEqual([2, 5]);
  });
});
