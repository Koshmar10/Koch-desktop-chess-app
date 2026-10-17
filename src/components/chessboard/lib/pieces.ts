import type { Square } from "../../../api/bindings/Square";
import { sameSquare } from "./square";
import type { PlacedPiece } from "./types";

/** The piece standing on `square`, if any. */
export const pieceAt = (
  pieces: readonly PlacedPiece[],
  square: Square,
): PlacedPiece | undefined =>
  pieces.find((piece) => sameSquare(piece.square, square));

/**
 * The piece with this id, if the position holds it. Analyzer findings name
 * pieces by id, so this is how every one of them is resolved; a missing id
 * means the findings and the position are a ply apart.
 */
export const pieceById = (
  pieces: readonly PlacedPiece[],
  id: number,
): PlacedPiece | undefined => pieces.find((piece) => piece.id === id);
