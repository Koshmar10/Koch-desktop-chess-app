import type { PieceKind, PlacedPiece } from "./types";

const BACK_RANK: PieceKind[] = ["rook", "knight", "bishop", "queen", "king", "bishop", "knight", "rook"];

export const STARTING_POSITION: PlacedPiece[] = [
  ...BACK_RANK.map((kind, col) => ({ kind, color: "black" as const, row: 0, col })),
  ...Array.from({ length: 8 }, (_, col) => ({ kind: "pawn" as const, color: "black" as const, row: 1, col })),
  ...Array.from({ length: 8 }, (_, col) => ({ kind: "pawn" as const, color: "white" as const, row: 6, col })),
  ...BACK_RANK.map((kind, col) => ({ kind, color: "white" as const, row: 7, col })),
];
