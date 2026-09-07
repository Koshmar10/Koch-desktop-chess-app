import type { PieceColor, PieceKind, PlacedPiece } from "./types";

const STARTING_COMPOSITION: Record<PieceKind, number> = {
  pawn: 8,
  rook: 2,
  knight: 2,
  bishop: 2,
  queen: 1,
  king: 1,
};

const opponentOf = (color: PieceColor): PieceColor =>
  color === "white" ? "black" : "white";

// Pieces `color` has captured, inferred from what's missing on the board
// relative to a full starting set. There's no captured-pieces list from the
// backend (yet), so this reflects current material only — not the order
// captures actually happened in.
export const getTakenPieces = (
  pieces: PlacedPiece[],
  color: PieceColor,
): [PieceKind, PieceColor][] => {
  const opponent = opponentOf(color);

  const remaining: Partial<Record<PieceKind, number>> = {};
  for (const piece of pieces) {
    if (piece.color !== opponent) continue;
    remaining[piece.kind] = (remaining[piece.kind] ?? 0) + 1;
  }

  const taken: [PieceKind, PieceColor][] = [];
  for (const kind of Object.keys(STARTING_COMPOSITION) as PieceKind[]) {
    const missing = STARTING_COMPOSITION[kind] - (remaining[kind] ?? 0);
    for (let i = 0; i < missing; i++) {
      taken.push([kind, opponent]);
    }
  }
  return taken;
};
