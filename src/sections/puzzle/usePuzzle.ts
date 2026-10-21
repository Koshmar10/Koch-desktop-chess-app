import { useCallback, useEffect, useState } from "react";
import { getPuzzle } from "../../api/puzzle";
import type { ChessPuzzle } from "../../api/bindings/ChessPuzzle";

interface Loaded {
  // Which request this answers. Kept with the result, so loading and the
  // error are derived from it rather than set beside it, and a slow answer
  // to an older request can never be taken for the current one.
  request: number;
  puzzle: ChessPuzzle | null;
  error: string | null;
}

// The request number before anything has been asked for.
const NOT_REQUESTED = 0;

/**
 * A puzzle from Lichess, once `ensureLoaded` asks for one, and another
 * each time `refresh` is called. Used by `PuzzleProvider`, above the
 * routes, so nothing is fetched until the puzzle screen first opens.
 *
 * The last puzzle that loaded stays until the next one arrives, so asking
 * for a new one shows a spinner over the old board rather than an empty
 * one. The same shape as the analyzer's `useGameReplay`.
 */
export const usePuzzle = () => {
  const [request, setRequest] = useState(NOT_REQUESTED);
  const [loaded, setLoaded] = useState<Loaded | null>(null);

  // Every fetch is a different random puzzle, and Strict Mode runs this
  // twice in development. Only the last run's answer is used, so the board
  // never loads one puzzle and then jumps to another.
  useEffect(() => {
    if (request === NOT_REQUESTED) return;
    let isCurrent = true;
    getPuzzle()
      .then((puzzle) => {
        if (!isCurrent) return;
        // DEBUG: remove before the PR.
        console.log(
          "puzzle solution:",
          puzzle.positions
            .slice(puzzle.start + 1)
            .map((position) => `${position.san} (${position.uci})`),
        );
        setLoaded({ request, puzzle, error: null });
      })
      .catch((err) => {
        if (!isCurrent) return;
        // A failed refresh leaves the last puzzle where it was.
        setLoaded((previous) => ({
          request,
          puzzle: previous?.puzzle ?? null,
          error: String(err),
        }));
      });
    return () => {
      isCurrent = false;
    };
  }, [request]);

  // Asks for the first puzzle unless one has been asked for already, so
  // calling it again, as Strict Mode does, fetches nothing more. The same
  // function every render, so the screen's effect can depend on it.
  const ensureLoaded = useCallback(
    () =>
      setRequest((previous) =>
        previous === NOT_REQUESTED ? NOT_REQUESTED + 1 : previous,
      ),
    [],
  );

  // Loading until the first answer, including before anything's asked:
  // the only one reading it is the screen, which asks as soon as it opens.
  const isForThisRequest = loaded !== null && loaded.request === request;
  return {
    puzzle: loaded?.puzzle ?? null,
    error: isForThisRequest ? loaded.error : null,
    isLoading: !isForThisRequest,
    ensureLoaded,
    refresh: () => setRequest((previous) => previous + 1),
  };
};
