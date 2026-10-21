import { useCallback, useEffect, useState } from "react";
import { getPuzzle } from "../../api/puzzle";
import type { ChessPuzzle } from "../../api/bindings/ChessPuzzle";

const NOT_REQUESTED = 0;

// Kept with the request it answers, so a slow answer to an older request is
// never taken for the current one.
interface PuzzleResult {
  request: number;
  puzzle: ChessPuzzle | null;
  error: string | null;
}

/** Same shape as the analyzer's `useGameReplay`: the old puzzle stays until the next one arrives. */
export const usePuzzle = () => {
  const [request, setRequest] = useState(NOT_REQUESTED);
  const [result, setResult] = useState<PuzzleResult | null>(null);

  useEffect(() => {
    if (request === NOT_REQUESTED) return;
    let isCurrent = true;
    getPuzzle()
      .then((puzzle) => {
        if (isCurrent) setResult({ request, puzzle, error: null });
      })
      .catch((err) => {
        if (!isCurrent) return;
        setResult((previous) => ({
          request,
          puzzle: previous?.puzzle ?? null,
          error: String(err),
        }));
      });
    return () => {
      isCurrent = false;
    };
  }, [request]);

  const ensureLoaded = useCallback(
    () =>
      setRequest((previous) =>
        previous === NOT_REQUESTED ? NOT_REQUESTED + 1 : previous,
      ),
    [],
  );

  const isForThisRequest = result !== null && result.request === request;
  return {
    puzzle: result?.puzzle ?? null,
    error: isForThisRequest ? result.error : null,
    isLoading: !isForThisRequest,
    ensureLoaded,
    refresh: () => setRequest((previous) => previous + 1),
  };
};
