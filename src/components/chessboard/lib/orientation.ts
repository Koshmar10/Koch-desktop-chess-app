import { BOARD_SIZE } from "./constants";

// Flips a [row, col] pair across both axes when `isFlipped`. Used at every
// boundary between the board's absolute (rank/file) coordinates and
// screen/grid coordinates — this transform is its own inverse (flipping
// twice is the identity), so the same function converts both directions:
// domain -> screen for rendering, screen -> domain for click/drag hit-testing.
export const flipCoords = (
  row: number,
  col: number,
  isFlipped: boolean,
): [number, number] =>
  isFlipped
    ? [BOARD_SIZE - 1 - row, BOARD_SIZE - 1 - col]
    : [row, col];

// Value equality for the [row, col]/[rank, file] tuples this whole board
// subsystem uses (ChessboardContext's `selectedSquare`, PieceLayer's
// `dragFrom`, etc.) — replaces the scattered `a?.[0] === b[0] && a?.[1] ===
// b[1]` checks with one named comparison, either side can be null.
export const squaresEqual = (
  a: [number, number] | null,
  b: [number, number] | null,
): boolean => a !== null && b !== null && a[0] === b[0] && a[1] === b[1];
