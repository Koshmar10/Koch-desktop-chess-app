import { useEffect, useState } from "react";
import { loadGameReplay } from "../../api/analyzer";
import type { GameReplay } from "../../api/bindings/GameReplay";

interface Loaded {
  // Which game this result belongs to. Kept with the result rather than
  // reset when the id changes, so loading/error are derived instead of
  // set from the effect — and a slow response for a previous game can
  // never be shown as the current one.
  gameId: number;
  replay: GameReplay | null;
  error: string | null;
}

/** Loads a saved game for the analyzer. `null` means sandbox: nothing to load. */
export const useGameReplay = (gameId: number | null) => {
  const [loaded, setLoaded] = useState<Loaded | null>(null);

  useEffect(() => {
    if (gameId === null) return;
    let isCurrent = true;
    loadGameReplay(gameId)
      .then((replay) => {
        if (isCurrent) setLoaded({ gameId, replay, error: null });
      })
      .catch((err) => {
        if (isCurrent) setLoaded({ gameId, replay: null, error: String(err) });
      });
    return () => {
      isCurrent = false;
    };
  }, [gameId]);

  const isForThisGame = loaded !== null && loaded.gameId === gameId;
  return {
    replay: isForThisGame ? loaded.replay : null,
    error: isForThisGame ? loaded.error : null,
    isLoading: gameId !== null && !isForThisGame,
  };
};
