export interface ArrowData {
  from: [number, number];
  to: [number, number];
  color?: string;
  type: "engine" | "user" | "ghost";
}

export type PieceColor = "white" | "black";
export type PieceKind = "pawn" | "knight" | "bishop" | "rook" | "queen" | "king";

export interface PlacedPiece {
  kind: PieceKind;
  color: PieceColor;
  row: number;
  col: number;
}
