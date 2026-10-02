import { invoke } from "@tauri-apps/api/core";
import type { GameReplay } from "./bindings/GameReplay";

// A saved game rebuilt position by position — pieces, the move that led
// to each, and that position's findings, all from one backend replay so
// the piece ids the findings name are the ones on the board.
export const loadGameReplay = (gameId: number): Promise<GameReplay> =>
  invoke<GameReplay>("load_game_replay", { gameId });
