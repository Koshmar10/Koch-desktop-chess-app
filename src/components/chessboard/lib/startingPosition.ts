import type { PieceKind, PlacedPiece } from "./types";

const BACK_RANK: PieceKind[] = [
  "rook",
  "knight",
  "bishop",
  "queen",
  "king",
  "bishop",
  "knight",
  "rook",
];

// Ids only need to be unique within this locally-built starting position —
// once a real game starts, pieces come from the backend with their own ids.
const blackBackRank: PlacedPiece[] = BACK_RANK.map((kind, file) => ({
  id: file,
  kind,
  color: "black",
  square: { rank: 0, file },
}));
const blackPawns: PlacedPiece[] = Array.from({ length: 8 }, (_, file) => ({
  id: 8 + file,
  kind: "pawn",
  color: "black",
  square: { rank: 1, file },
}));
const whitePawns: PlacedPiece[] = Array.from({ length: 8 }, (_, file) => ({
  id: 16 + file,
  kind: "pawn",
  color: "white",
  square: { rank: 6, file },
}));
const whiteBackRank: PlacedPiece[] = BACK_RANK.map((kind, file) => ({
  id: 24 + file,
  kind,
  color: "white",
  square: { rank: 7, file },
}));

export const STARTING_POSITION: PlacedPiece[] = [
  ...blackBackRank,
  ...blackPawns,
  ...whitePawns,
  ...whiteBackRank,
];
