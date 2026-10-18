import { describe, expect, it } from "vitest";
import { coordinateFromMouseEvent, squareAtPoint } from "../coordinate";

const SQUARE_SIZE_PX = 72;
const ORIGIN = { left: 100, top: 50 };

describe("coordinateFromMouseEvent", () => {
  it("reads clientX/clientY off the event", () => {
    const event = { clientX: 12, clientY: 34 } as MouseEvent;

    expect(coordinateFromMouseEvent(event)).toEqual({ x: 12, y: 34 });
  });
});

describe("squareAtPoint", () => {
  it("measures from the board's corner, not the window's", () => {
    const point = { x: ORIGIN.left + 1, y: ORIGIN.top + 1 };

    expect(squareAtPoint(point, ORIGIN, SQUARE_SIZE_PX, false)).toEqual({
      rank: 0,
      file: 0,
    });
  });

  it("picks the square the point falls inside", () => {
    // Middle of the cell at row 2, col 3.
    const point = {
      x: ORIGIN.left + 3.5 * SQUARE_SIZE_PX,
      y: ORIGIN.top + 2.5 * SQUARE_SIZE_PX,
    };

    expect(squareAtPoint(point, ORIGIN, SQUARE_SIZE_PX, false)).toEqual({
      rank: 2,
      file: 3,
    });
  });

  it("undoes the flip, so the result is a board square", () => {
    const topLeft = { x: ORIGIN.left + 1, y: ORIGIN.top + 1 };

    expect(squareAtPoint(topLeft, ORIGIN, SQUARE_SIZE_PX, true)).toEqual({
      rank: 7,
      file: 7,
    });
  });

  it("is null off the board on any side", () => {
    const boardEnd = 8 * SQUARE_SIZE_PX;

    expect(
      squareAtPoint(
        { x: ORIGIN.left - 1, y: ORIGIN.top },
        ORIGIN,
        SQUARE_SIZE_PX,
        false,
      ),
    ).toBeNull();
    expect(
      squareAtPoint(
        { x: ORIGIN.left, y: ORIGIN.top - 1 },
        ORIGIN,
        SQUARE_SIZE_PX,
        false,
      ),
    ).toBeNull();
    expect(
      squareAtPoint(
        { x: ORIGIN.left + boardEnd, y: ORIGIN.top },
        ORIGIN,
        SQUARE_SIZE_PX,
        false,
      ),
    ).toBeNull();
    expect(
      squareAtPoint(
        { x: ORIGIN.left, y: ORIGIN.top + boardEnd },
        ORIGIN,
        SQUARE_SIZE_PX,
        false,
      ),
    ).toBeNull();
  });
});
