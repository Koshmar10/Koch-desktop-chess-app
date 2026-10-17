import type { Square } from "../../../api/bindings/Square";
import { toScreen } from "../lib/orientation";
import type { Coordinate } from "./coordinate";

/** The pixel centre of a board square, measured from the board's corner. */
export const getSquareCenter = (
  square: Square,
  isFlipped: boolean,
  squareSize: number,
): Coordinate => {
  const [row, col] = toScreen(square, isFlipped);
  return {
    x: col * squareSize + squareSize / 2,
    y: row * squareSize + squareSize / 2,
  };
};
