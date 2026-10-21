import { useCallback, useState, type ReactNode } from "react";
import { PuzzleContext } from "./PuzzleContext";
import { puzzlePoints } from "./scoring";
import { solveFor } from "./solve";
import type { Solve } from "./types";
import { usePuzzle } from "./usePuzzle";

interface PuzzleProviderProps {
  children: ReactNode;
}

const PuzzleProvider = ({ children }: PuzzleProviderProps) => {
  const { puzzle, error, isLoading, ensureLoaded, refresh } = usePuzzle();
  const [storedSolve, setStoredSolve] = useState<Solve | null>(null);
  const [points, setPoints] = useState(0);
  const [streak, setStreak] = useState(0);

  // Asked of what's stored rather than reset when a new puzzle arrives:
  // progress stored for an older puzzle just isn't this one's.
  const solve = puzzle === null ? null : solveFor(storedSolve, puzzle);

  // The same function until the puzzle changes, so the screen's timers
  // that depend on it don't restart whenever the points or streak do.
  const updateSolve = useCallback(
    (change: (solve: Solve) => Solve) => {
      if (puzzle === null) return;
      setStoredSolve((stored) => change(solveFor(stored, puzzle)));
    },
    [puzzle],
  );

  const recordSolve = (rating: number, mistakes: number) => {
    const earned = puzzlePoints(rating, mistakes, streak);
    setPoints(points + earned);
    // A puzzle with mistakes already ended the streak at its first one.
    setStreak(mistakes === 0 ? streak + 1 : 0);
    return earned;
  };

  return (
    <PuzzleContext.Provider
      value={{
        puzzle,
        solve,
        error,
        isLoading,
        ensureLoaded,
        newPuzzle: refresh,
        updateSolve,
        points,
        streak,
        recordSolve,
        breakStreak: () => setStreak(0),
      }}
    >
      {children}
    </PuzzleContext.Provider>
  );
};

export default PuzzleProvider;
