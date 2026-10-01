import type { Square } from "../../../api/bindings/Square";
import { BOARD_SIZE } from "./constants";

export const FILES = ["a", "b", "c", "d", "e", "f", "g", "h"];

// Rank 0 is the *eighth* rank — the board's coordinates run top-down from
// Black's back rank, the way the pieces array is laid out, while algebraic
// notation counts bottom-up from White's. Everything that prints a square
// to a human goes through here rather than redoing that inversion.
export const squareName = ({ rank, file }: Square): string =>
  `${FILES[file]}${BOARD_SIZE - rank}`;
