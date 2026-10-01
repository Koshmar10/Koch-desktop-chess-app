import { squareName } from "../../../components/chessboard/lib/squareName";
import type {
  PieceKind,
  PlacedPiece,
} from "../../../components/chessboard/lib/types";

// Standard English piece letters. A pawn has none — "e5" *is* how a pawn
// is named, and "Pe5" reads as a typo to anyone who plays.
const PIECE_LETTER: Record<PieceKind, string> = {
  king: "K",
  queen: "Q",
  rook: "R",
  bishop: "B",
  knight: "N",
  pawn: "",
};

/**
 * Every analyzer finding identifies pieces by id, never by square — a pin
 * is three ids, a fork is a forker plus its targets. Rendering any of them
 * therefore needs the position's piece list to resolve against, which is
 * why the findings panel takes `pieces` rather than being able to render
 * from `PositionFindings` alone.
 *
 * Returns null for an id with no matching piece rather than a placeholder:
 * that only happens when the findings and the board are from different
 * plies, and silently printing "?" would hide the desync.
 */
export const pieceLabel = (
  id: number,
  pieces: PlacedPiece[],
): string | null => {
  const piece = pieces.find((p) => p.id === id);
  if (!piece) return null;
  return `${PIECE_LETTER[piece.kind]}${squareName(piece.square)}`;
};
