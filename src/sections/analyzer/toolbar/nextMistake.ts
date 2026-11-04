import type { MoveQuality } from "../../../api/bindings/MoveQuality";
import type { PieceColor } from "../../../api/bindings/PieceColor";

const MISTAKES: ReadonlySet<MoveQuality> = new Set(["Mistake", "Blunder"]);

// Ply 0 is White's first move: a saved game always starts from the
// standard position.
const moverOf = (ply: number): PieceColor =>
  ply % 2 === 0 ? "white" : "black";

// Whether the move at `ply` was a mistake or blunder by `side` (either
// side when null).
const isMistake = (
  quality: MoveQuality | null,
  ply: number,
  side: PieceColor | null,
) =>
  quality !== null &&
  MISTAKES.has(quality) &&
  (side === null || moverOf(ply) === side);

/**
 * The first ply after `afterPly` graded a mistake or a blunder, or null if
 * there's none. `qualities[ply]` is the grade of the move played at `ply`.
 * With `side`, only that side's moves count — in a game you played, the
 * mistakes worth jumping to are your own.
 */
export const nextMistakePly = (
  qualities: readonly (MoveQuality | null)[],
  afterPly: number,
  side: PieceColor | null,
): number | null => {
  const found = qualities.findIndex(
    (quality, ply) => ply > afterPly && isMistake(quality, ply, side),
  );
  return found === -1 ? null : found;
};

/** The last such ply before `beforePly`, or null; the way back. */
export const previousMistakePly = (
  qualities: readonly (MoveQuality | null)[],
  beforePly: number,
  side: PieceColor | null,
): number | null => {
  // A loop rather than `findLastIndex`, which the configured TS lib
  // doesn't have.
  for (let ply = Math.min(beforePly, qualities.length) - 1; ply >= 0; ply--) {
    if (isMistake(qualities[ply], ply, side)) return ply;
  }
  return null;
};
