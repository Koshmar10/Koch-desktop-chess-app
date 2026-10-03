import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { EngineSnapshot } from "./bindings/EngineSnapshot";
import type { GameReplay } from "./bindings/GameReplay";

// A saved game rebuilt position by position — pieces, the move that led
// to each, and that position's findings, all from one backend replay so
// the piece ids the findings name are the ones on the board.
export const loadGameReplay = (gameId: number): Promise<GameReplay> =>
  invoke<GameReplay>("load_game_replay", { gameId });

// ---- Live engine (KOCH-10) ----

export const refreshLiveEngineSession = (
  gameId: number | null,
): Promise<void> => invoke("refresh_live_engine_session", { gameId });

/**
 * Searches the position `moves` lead to from the start. `positionKey` comes
 * back on every snapshot of this search.
 */
export const updateLiveEnginePosition = (
  positionKey: string,
  moves: string[],
): Promise<void> =>
  invoke("update_live_engine_position", {
    startingPosition: null,
    moves,
    positionKey,
  });

export const stopLiveEngine = (): Promise<void> => invoke("stop_live_engine");

export const onLiveEngineSnapshot = (
  handler: (snapshot: EngineSnapshot) => void,
): Promise<UnlistenFn> =>
  listen<EngineSnapshot>("pvline", (event) => handler(event.payload));
