import type { FormAxis, PieceAccuracy } from "./types";

// Stand-ins for the form guide and the army gauges until the backend
// reports accuracy per skill and per piece. Each block that shows them
// says so, so they're never read as the player's own numbers.

export const SAMPLE_FORM: FormAxis[] = [
  { skill: "Opening", recent: 78, allTime: 70 },
  { skill: "Middlegame", recent: 66, allTime: 64 },
  { skill: "Endgame", recent: 58, allTime: 62 },
  { skill: "Time", recent: 54, allTime: 60 },
  { skill: "Defence", recent: 38, allTime: 52 },
  { skill: "Tactics", recent: 61, allTime: 58 },
];

export const SAMPLE_ARMY: PieceAccuracy[] = [
  { piece: "king", accuracy: 74 },
  { piece: "queen", accuracy: 52 },
  { piece: "rook", accuracy: 68 },
  { piece: "bishop", accuracy: 76 },
  { piece: "knight", accuracy: 88 },
  { piece: "pawn", accuracy: 81 },
];
