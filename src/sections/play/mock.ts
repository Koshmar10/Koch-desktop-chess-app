import type { PieceColor, PieceKind } from "../../components/chessboard/lib/types";

export const MOCK_WHITE_CAPTURES: [PieceKind, PieceColor][] = [
  ["pawn", "black"],
  ["pawn", "black"],
  ["knight", "black"],
];

export const MOCK_BLACK_CAPTURES: [PieceKind, PieceColor][] = [
  ["pawn", "white"],
];

export const MOCK_WHITE_CLOCK = "09:58";
export const MOCK_BLACK_CLOCK = "08:42";
