import { useEffect, useState } from "react";
import {
  loadGameReplay,
  onLiveEngineSnapshot,
  refreshLiveEngineSession,
} from "../../api/analyzer";
import type { EngineSnapshot } from "../../api/bindings/EngineSnapshot";
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

/**
 * The live engine's latest full set of lines, for whichever position it was
 * last searching — the caller checks its `position_key`.
 */
export const useLiveEngine = (): EngineSnapshot | null => {
  const [liveSnapshot, setLiveSnapshot] = useState<EngineSnapshot | null>(null);

  // Subscribed once, on mount. Without the `[]` it re-subscribed after every
  // render — and every snapshot is a render — dropping whatever arrived
  // while the old listener was gone and the new one not yet attached.
  useEffect(() => {
    const unlisten = onLiveEngineSnapshot(setLiveSnapshot);
    return () => {
      unlisten.then((stop) => stop());
    };
  }, []);

  return liveSnapshot;
};

/**
 * Loads a saved game for the analyzer, and keeps the live engine session
 * on that game. `null` means sandbox: nothing to load.
 *
 * `isAnalysisDone` turning true loads the game again, so a re-analysis
 * shows its new evals and grades once it finishes. Until the new load
 * arrives the old one stays on screen, rather than flashing a spinner.
 */
export const useGameReplay = (
  gameId: number | null,
  isAnalysisDone: boolean,
) => {
  const [loaded, setLoaded] = useState<Loaded | null>(null);

  // `isAnalysisDone` isn't read inside — it's a dependency only so the
  // load runs again when it changes.
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
  }, [gameId, isAnalysisDone]);

  useEffect(() => {
    refreshLiveEngineSession(gameId).catch(console.error);
  }, [gameId]);

  const isForThisGame = loaded !== null && loaded.gameId === gameId;
  return {
    replay: isForThisGame ? loaded.replay : null,
    error: isForThisGame ? loaded.error : null,
    isLoading: gameId !== null && !isForThisGame,
  };
};
