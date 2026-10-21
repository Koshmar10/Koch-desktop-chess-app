import { describe, expect, it } from "vitest";
import { TONE_SQUARE_COLOR } from "../../../components/chessboard/lib/highlightTones";
import { correctMoveMask, wrongMoveMask } from "../masks";

const E2 = { rank: 6, file: 4 };
const E4 = { rank: 4, file: 4 };
const MOVE = { from: E2, to: E4 };

describe("wrongMoveMask", () => {
  it("fills both of the move's squares red", () => {
    expect(wrongMoveMask(MOVE).squares).toEqual([
      { square: E2, color: TONE_SQUARE_COLOR.danger },
      { square: E4, color: TONE_SQUARE_COLOR.danger },
    ]);
  });
});

describe("correctMoveMask", () => {
  it("fills both of the move's squares green", () => {
    expect(correctMoveMask(MOVE).squares).toEqual([
      { square: E2, color: TONE_SQUARE_COLOR.good },
      { square: E4, color: TONE_SQUARE_COLOR.good },
    ]);
  });

  it("has an id of its own, so the two never share a key", () => {
    expect(correctMoveMask(MOVE).id).not.toBe(wrongMoveMask(MOVE).id);
  });
});
