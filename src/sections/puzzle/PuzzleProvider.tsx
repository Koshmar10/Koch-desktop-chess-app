import { useCallback, useState, type ReactNode } from "react";
import { PuzzleContext } from "./PuzzleContext";
import { progressFor } from "./progress";
import { puzzlePoints } from "./scoring";
import type { PuzzleProgress } from "./types";
import { usePuzzle } from "./usePuzzle";

interface PuzzleProviderProps {
  children: ReactNode;
}

const PuzzleProvider = ({ children }: PuzzleProviderProps) => {
  const { puzzle, error, isLoading, ensureLoaded, refresh } = usePuzzle();
  const [storedProgress, setStoredProgress] = useState<PuzzleProgress | null>(
    null,
  );
  const [points, setPoints] = useState(0);
  const [streak, setStreak] = useState(0);

  const progress = puzzle === null ? null : progressFor(storedProgress, puzzle);

  // Stable per puzzle, so the screen's timers don't restart when the points
  // change.
  const updateProgress = useCallback(
    (change: (progress: PuzzleProgress) => PuzzleProgress) => {
      if (puzzle === null) return;
      setStoredProgress((stored) => change(progressFor(stored, puzzle)));
    },
    [puzzle],
  );

  const recordSolvedPuzzle = (rating: number, mistakes: number) => {
    const earnedPoints = puzzlePoints(rating, mistakes, streak);
    setPoints(points + earnedPoints);
    setStreak(mistakes === 0 ? streak + 1 : 0);
    return earnedPoints;
  };

  return (
    <PuzzleContext.Provider
      value={{
        puzzle,
        progress,
        error,
        isLoading,
        ensureLoaded,
        requestNewPuzzle: refresh,
        updateProgress,
        points,
        streak,
        recordSolvedPuzzle,
        breakStreak: () => setStreak(0),
      }}
    >
      {children}
    </PuzzleContext.Provider>
  );
};

export default PuzzleProvider;
