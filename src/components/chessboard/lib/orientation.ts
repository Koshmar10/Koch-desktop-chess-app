import type { Square } from "../../../api/bindings/Square";
import { BOARD_SIZE } from "./constants";

/**
 * A cell of the on-screen grid: row 0 is the top row as drawn, col 0 the
 * left column. Distinct from a `Square`, which is where something is on
 * the board itself — the two only line up while the board isn't flipped.
 * Kept as its own type so the compiler, not the reader, catches a board
 * square passed where a screen cell was meant.
 */
export type ScreenCell = [row: number, col: number];

const mirror = (index: number): number => BOARD_SIZE - 1 - index;

// Flipping mirrors both axes. That's its own inverse, so `toScreen` and
// `fromScreen` are the same arithmetic — they differ only in which type
// goes in and which comes out.

/** Where a board square is drawn: for rendering. */
export const toScreen = (
  { rank, file }: Square,
  isFlipped: boolean,
): ScreenCell => (isFlipped ? [mirror(rank), mirror(file)] : [rank, file]);

/** Which board square is drawn at a screen cell: for hit-testing. */
export const fromScreen = (
  [row, col]: ScreenCell,
  isFlipped: boolean,
): Square =>
  isFlipped
    ? { rank: mirror(row), file: mirror(col) }
    : { rank: row, file: col };
