import type { Square } from "../../../api/bindings/Square";
import { sameSquare } from "./square";
import type { PlacedPiece } from "./types";

export const applyMove = (
  pieces: PlacedPiece[],
  from: Square,
  to: Square,
): PlacedPiece[] =>
  pieces
    .filter((piece) => !sameSquare(piece.square, to))
    .map((piece) =>
      sameSquare(piece.square, from) ? { ...piece, square: to } : piece,
    );
