import type { PieceAccuracy } from "./types";

// A stand-in for the army gauges until the backend reports accuracy per
// piece. The block says so, so it's never read as the player's own numbers.

export const SAMPLE_ARMY: PieceAccuracy[] = [
  { piece: "king", accuracy: 74 },
  { piece: "queen", accuracy: 52 },
  { piece: "rook", accuracy: 68 },
  { piece: "bishop", accuracy: 76 },
  { piece: "knight", accuracy: 88 },
  { piece: "pawn", accuracy: 81 },
];
