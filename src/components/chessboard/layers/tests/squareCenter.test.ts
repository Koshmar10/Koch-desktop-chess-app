import { describe, expect, it } from "vitest";
import { getSquareCenter } from "../squareCenter";

const SQUARE_SIZE = 72;

describe("getSquareCenter", () => {
  it("centers on the square using unflipped board coordinates", () => {
    const center = getSquareCenter([0, 0], false, SQUARE_SIZE);

    expect(center).toEqual({ x: SQUARE_SIZE / 2, y: SQUARE_SIZE / 2 });
  });

  it("mirrors both axes when the board is flipped", () => {
    const center = getSquareCenter([0, 0], true, SQUARE_SIZE);

    expect(center).toEqual({
      x: 7 * SQUARE_SIZE + SQUARE_SIZE / 2,
      y: 7 * SQUARE_SIZE + SQUARE_SIZE / 2,
    });
  });

  it("scales with square size", () => {
    const center = getSquareCenter([2, 3], false, SQUARE_SIZE);

    expect(center).toEqual({
      x: 3 * SQUARE_SIZE + SQUARE_SIZE / 2,
      y: 2 * SQUARE_SIZE + SQUARE_SIZE / 2,
    });
  });
});
