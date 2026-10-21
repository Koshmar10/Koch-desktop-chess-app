import { createContext, useContext } from "react";
import type { ChessPuzzle } from "../../api/bindings/ChessPuzzle";
import type { Solve } from "./types";

/**
 * What outlives the puzzle screen, as `AnalyzerProvider` keeps what
 * outlives the analyzer: the puzzle on the board, how far its solve has
 * got, and this session's points and streak. Leaving the screen and coming
 * back resumes the puzzle instead of fetching a new one. All of it resets
 * when the app restarts; saving across restarts is KOCH-25's.
 */
export interface PuzzleContextValue {
  // The last puzzle that loaded. Null until the first one does.
  puzzle: ChessPuzzle | null;
  // That puzzle's progress; null exactly when `puzzle` is.
  solve: Solve | null;
  error: string | null;
  isLoading: boolean;
  // Asks for the first puzzle, if none has been asked for yet. The screen
  // calls it when it opens, so nothing's fetched before that.
  ensureLoaded: () => void;
  newPuzzle: () => void;
  // Changes the current puzzle's progress. Takes the progress as it is
  // when the change lands, not as it was when the change was made, so a
  // reply landing on a timer can't undo a move made in between.
  updateSolve: (change: (solve: Solve) => Solve) => void;
  points: number;
  // Puzzles solved in a row without a mistake or a skip.
  streak: number;
  // Adds a solved puzzle's points, extends or ends the streak, and returns
  // what the puzzle was worth.
  recordSolve: (rating: number, mistakes: number) => number;
  // For a mistake, or a puzzle left unsolved for a new one.
  breakStreak: () => void;
}

export const PuzzleContext = createContext<PuzzleContextValue | null>(null);

export const usePuzzleContext = (): PuzzleContextValue => {
  const ctx = useContext(PuzzleContext);
  if (!ctx) {
    throw new Error("usePuzzleContext must be used inside <PuzzleProvider>");
  }
  return ctx;
};
