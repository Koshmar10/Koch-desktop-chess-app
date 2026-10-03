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
 * Loads a saved game for the analyzer, and keeps the live engine session
 * on that game. `null` means sandbox: nothing to load.
 */
export const useGameReplay = (gameId: number | null) => {
  const [loaded, setLoaded] = useState<Loaded | null>(null);
  // The live engine's latest full set of lines, for whichever position it
  // was last searching — the caller checks its `position_key`.
  const [liveSnapshot, setLiveSnapshot] = useState<EngineSnapshot | null>(null);

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

  useEffect(() => {
    refreshLiveEngineSession(gameId).catch(console.error);
  }, [gameId]);

  useEffect(() => {
    const unlisten = onLiveEngineSnapshot(setLiveSnapshot);
    return () => {
      unlisten.then((stop) => stop());
    };
  }, []);

  const isForThisGame = loaded !== null && loaded.gameId === gameId;
  return {
    replay: isForThisGame ? loaded.replay : null,
    error: isForThisGame ? loaded.error : null,
    isLoading: gameId !== null && !isForThisGame,
    liveSnapshot,
  };
};
