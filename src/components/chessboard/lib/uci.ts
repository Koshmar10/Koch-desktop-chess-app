import type { Square } from "../../../api/bindings/Square";
import { BOARD_SIZE } from "./constants";
import { FILES } from "./squareName";

/**
 * "e4" → { rank: 4, file: 4 }, or null for anything that isn't a square.
 * The inverse of `squareName`: rank 0 is the eighth rank, so the digit
 * has to be flipped on the way in exactly as it's flipped on the way out.
 */
export const parseSquare = (name: string): Square | null => {
  const file = FILES.indexOf(name[0]);
  const rankDigit = Number(name[1]);
  if (name.length !== 2 || file === -1 || !Number.isInteger(rankDigit)) {
    return null;
  }
  if (rankDigit < 1 || rankDigit > BOARD_SIZE) return null;
  return { rank: BOARD_SIZE - rankDigit, file };
};

/**
 * The two squares of a UCI move — "e2e4", or "e7e8q" with the promotion
 * piece ignored, since an arrow only needs where it starts and ends.
 * Null for anything malformed, so a bad line from the engine draws
 * nothing rather than an arrow to the wrong corner.
 */
export const parseUciMove = (
  uci: string,
): { from: Square; to: Square } | null => {
  const from = parseSquare(uci.slice(0, 2));
  const to = parseSquare(uci.slice(2, 4));
  return from && to ? { from, to } : null;
};
