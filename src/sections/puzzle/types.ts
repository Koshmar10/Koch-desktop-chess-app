import type { LastMove } from "../../api/bindings/LastMove";
import type { PieceColor } from "../../api/bindings/PieceColor";
import type { PlacedPiece } from "../../components/chessboard/lib/types";

export interface WrongMove {
  move: LastMove;
  // Built on the frontend: a wrong move is never played, so the backend has
  // no position for it.
  pieces: PlacedPiece[];
}

export interface Mistake {
  piece: PlacedPiece;
  move: LastMove;
  isCapture: boolean;
}

export type SolvedCardState = "hidden" | "shown" | "dismissed";

// Plies are indexes into the puzzle's `positions`.
export interface PuzzleProgress {
  puzzleId: string;
  currentPly: number;
  viewedPly: number;
  turn: PieceColor;
  mistakes: Mistake[];
  earnedPoints: number;
  solvedCard: SolvedCardState;
}
