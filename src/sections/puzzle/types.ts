import type { LastMove } from "../../api/bindings/LastMove";
import type { PieceColor } from "../../api/bindings/PieceColor";
import type { PlacedPiece } from "../../components/chessboard/lib/types";

/**
 * A legal move that wasn't the solution's, shown for a moment and then
 * taken back. `pieces` is the position it leads to, worked out on the
 * frontend: the move is never played, so the backend has no position for
 * it.
 */
export interface WrongMove {
  move: LastMove;
  pieces: PlacedPiece[];
}

/** A legal move the solver tried that wasn't the solution's. */
export interface Mistake {
  // The piece that moved, as it stood before the move.
  piece: PlacedPiece;
  move: LastMove;
  isCapture: boolean;
}

/**
 * How far a puzzle's solve has got. Kept by `PuzzleProvider`, above the
 * routes, so leaving the screen and coming back resumes it.
 */
export interface Solve {
  // Which puzzle this is the progress of.
  puzzleId: string;
  // An index into the puzzle's `positions`: moves are only played from
  // here.
  currentPly: number;
  // Whose move it is. Flips with every move played.
  turn: PieceColor;
  // Oldest first.
  mistakes: Mistake[];
  // What solving it added to the session's points; 0 until it's solved.
  earnedPoints: number;
}
