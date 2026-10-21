import type { ChessPuzzle } from "../../api/bindings/ChessPuzzle";
import { opponentOf } from "../../components/chessboard/lib/pieces";
import type { PuzzleProgress } from "./types";

export const newProgress = (puzzle: ChessPuzzle): PuzzleProgress => ({
  puzzleId: puzzle.id,
  // One position early, so the opponent's setup move plays first, as on
  // Lichess.
  currentPly: puzzle.start - 1,
  viewedPly: puzzle.start - 1,
  turn: opponentOf(puzzle.solver_color),
  mistakes: [],
  earnedPoints: 0,
  solvedCard: "hidden",
});

export const progressFor = (
  stored: PuzzleProgress | null,
  puzzle: ChessPuzzle,
): PuzzleProgress =>
  stored?.puzzleId === puzzle.id ? stored : newProgress(puzzle);
