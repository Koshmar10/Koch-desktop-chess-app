import type { ChessPuzzle } from "../../api/bindings/ChessPuzzle";
import { opponentOf } from "../../components/chessboard/lib/pieces";
import type { Solve } from "./types";

/**
 * A puzzle not yet started. It begins one position before the solver's,
 * with the opponent to play the move the puzzle answers, as on Lichess.
 */
export const startSolve = (puzzle: ChessPuzzle): Solve => ({
  puzzleId: puzzle.id,
  currentPly: puzzle.start - 1,
  turn: opponentOf(puzzle.turn),
  mistakes: [],
  earnedPoints: 0,
});

/**
 * The progress stored for `puzzle`, or a fresh start if what's stored
 * belongs to an older one. Asked rather than reset when a new puzzle
 * arrives, so there's no reset to forget.
 */
export const solveFor = (stored: Solve | null, puzzle: ChessPuzzle): Solve =>
  stored?.puzzleId === puzzle.id ? stored : startSolve(puzzle);
