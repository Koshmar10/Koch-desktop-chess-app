import type { PieceType } from "../../api/bindings/PieceType";

/** How accurately the player moves one kind of piece, 0–100. */
export interface PieceAccuracy {
  piece: PieceType;
  accuracy: number;
}
