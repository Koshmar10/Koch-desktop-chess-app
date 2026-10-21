import { createContext, useContext } from "react";
import type { ChessPuzzle } from "../../api/bindings/ChessPuzzle";
import type { PuzzleProgress } from "./types";

export interface PuzzleContextValue {
  puzzle: ChessPuzzle | null;
  progress: PuzzleProgress | null;
  error: string | null;
  isLoading: boolean;
  ensureLoaded: () => void;
  newPuzzle: () => void;
  updateProgress: (
    change: (progress: PuzzleProgress) => PuzzleProgress,
  ) => void;
  points: number;
  streak: number;
  // Returns the points the puzzle was worth.
  recordSolvedPuzzle: (rating: number, mistakes: number) => number;
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
