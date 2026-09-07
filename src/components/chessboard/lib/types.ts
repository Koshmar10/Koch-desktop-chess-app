export type { PieceColor } from "../../../api/bindings/PieceColor";
export type { PieceType as PieceKind } from "../../../api/bindings/PieceType";
export type { PieceView as PlacedPiece } from "../../../api/bindings/PieceView";

export interface ArrowData {
  from: [number, number];
  to: [number, number];
  color?: string;
  type: "engine" | "user" | "ghost";
}
