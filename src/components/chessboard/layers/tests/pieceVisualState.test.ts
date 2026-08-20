import { describe, expect, it } from "vitest";
import { getPieceVisualState } from "../pieceVisualState";

const SQUARE_SIZE = 72;

describe("getPieceVisualState", () => {
  it("places a resting piece on its grid cell at the board's square size", () => {
    const { className, style } = getPieceVisualState(
      false,
      null,
      3,
      5,
      SQUARE_SIZE,
    );

    expect(className).toContain("cursor-grab");
    expect(style).toMatchObject({
      gridRowStart: 4,
      gridColumnStart: 6,
      width: SQUARE_SIZE,
      height: SQUARE_SIZE,
    });
  });

  it("follows the cursor and enlarges while being dragged", () => {
    const dragPosition = { x: 100, y: 200 };
    const { className, style } = getPieceVisualState(
      true,
      dragPosition,
      3,
      5,
      SQUARE_SIZE,
    );

    expect(className).toContain("pointer-events-none");
    expect(style).toMatchObject({
      left: dragPosition.x - SQUARE_SIZE / 2,
      top: dragPosition.y - SQUARE_SIZE / 2,
      width: SQUARE_SIZE * 1.1,
      height: SQUARE_SIZE * 1.1,
    });
  });

  it("falls back to the resting state if dragged with no known position", () => {
    const { style } = getPieceVisualState(true, null, 3, 5, SQUARE_SIZE);

    expect(style).toMatchObject({
      gridRowStart: 4,
      gridColumnStart: 6,
    });
  });
});
