import type { Square } from "../../../api/bindings/Square";

/**
 * Value equality for squares. Two `Square` objects are never `===` unless
 * they're the same object, so every comparison goes through here rather
 * than spelling out `a.rank === b.rank && a.file === b.file` again. False
 * whenever either side is missing — "no square" never matches anything.
 */
export const sameSquare = (a: Square | null, b: Square | null): boolean =>
  a !== null && b !== null && a.rank === b.rank && a.file === b.file;

// Parity is the same in board and screen coordinates — flipping maps
// (r, f) to (7−r, 7−f), which changes the sum by an even number.
export const isDarkSquare = ({ rank, file }: Square): boolean =>
  (rank + file) % 2 === 1;
