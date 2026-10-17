import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { EngineSnapshot } from "./bindings/EngineSnapshot";
import type { GameReplay } from "./bindings/GameReplay";
import type { LiveEngineSettings } from "./bindings/LiveEngineSettings";
import type { PieceType } from "./bindings/PieceType";
import type { SandboxMoved } from "./bindings/SandboxMoved";
import type { SandboxStarted } from "./bindings/SandboxStarted";
import type { Square } from "./bindings/Square";

// A saved game rebuilt position by position — pieces, the move that led
// to each, and that position's findings, all from one backend replay so
// the piece ids the findings name are the ones on the board.
export const loadGameReplay = (gameId: number): Promise<GameReplay> =>
  invoke<GameReplay>("load_game_replay", { gameId });

// ---- Sandbox ----

/**
 * Starts a fresh sandbox on `fen` — the standard start when null —
 * replacing any open one. Resolves to its positions (just the start,
 * `positions[0]`, the same shape a replay uses) and what can move.
 */
export const sandboxNew = (fen: string | null): Promise<SandboxStarted> =>
  invoke<SandboxStarted>("sandbox_new", { fen });

/**
 * Starts a fresh sandbox on a pasted FEN, or on a PGN with its moves already
 * played, replacing any open one. Rejects with a message to show the user
 * if it reads as neither.
 */
export const sandboxLoad = (notation: string): Promise<SandboxStarted> =>
  invoke<SandboxStarted>("sandbox_load", { notation });

/**
 * Takes back the sandbox's last move. Resolves to the whole sandbox, the
 * same shape a fresh one comes back in.
 */
export const sandboxUndo = (): Promise<SandboxStarted> =>
  invoke<SandboxStarted>("sandbox_undo");

/**
 * Plays a move in the open sandbox. Resolves to the position it leads to —
 * only that one, to append to the list — and what can move from there;
 * rejects on an illegal move. With no `promotion`, a pawn queens.
 */
export const sandboxMove = (
  from: Square,
  to: Square,
  promotion: PieceType | null,
): Promise<SandboxMoved> =>
  invoke<SandboxMoved>("sandbox_move", { from, to, promotion });

// ---- Live engine (KOCH-10) ----

export const refreshLiveEngineSession = (
  gameId: number | null,
): Promise<void> => invoke("refresh_live_engine_session", { gameId });

/**
 * Searches the position `moves` lead to from `startingPosition` (a FEN; the
 * standard start when null). `positionKey` comes back on every snapshot of
 * this search.
 */
export const updateLiveEnginePosition = (
  positionKey: string,
  startingPosition: string | null,
  moves: string[],
): Promise<void> =>
  invoke("update_live_engine_position", {
    startingPosition,
    moves,
    positionKey,
  });

export const stopLiveEngine = (): Promise<void> => invoke("stop_live_engine");

/** What the session is running with — Stockfish's defaults before any change. */
export const getLiveEngineSettings = (): Promise<LiveEngineSettings> =>
  invoke<LiveEngineSettings>("get_live_engine_settings");

/**
 * Applies new options to the running session, without saving them. The
 * search stops for them; re-send the position to pick it back up.
 */
export const applyLiveEngineSettings = (
  settings: LiveEngineSettings,
): Promise<void> => invoke("apply_live_engine_settings", { settings });

export const onLiveEngineSnapshot = (
  handler: (snapshot: EngineSnapshot) => void,
): Promise<UnlistenFn> =>
  listen<EngineSnapshot>("pvline", (event) => handler(event.payload));
