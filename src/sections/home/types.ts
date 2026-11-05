import type { PieceType } from "../../api/bindings/PieceType";

/** One spoke of the form guide: a skill scored 0–100, lately and overall. */
export interface FormAxis {
  skill: string;
  recent: number;
  allTime: number;
}

/** How accurately the player moves one kind of piece, 0–100. */
export interface PieceAccuracy {
  piece: PieceType;
  accuracy: number;
}
