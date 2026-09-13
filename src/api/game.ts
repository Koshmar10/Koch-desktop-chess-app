import { invoke } from "@tauri-apps/api/core"
import { GameStateView } from "./bindings/GameStateView"
import { PieceType } from "./bindings/PieceType"
import { Square } from "./bindings/Square"
import { PieceColor } from "./bindings/PieceColor"
import { GameCreateResponse } from "./bindings/GameCreateResponse";
import { TerminationReason } from "./bindings/TerminationReason";
import { GameResult } from "./bindings/GameResult";
import { TimeControl } from "./bindings/TimeControl";
import { GameSummary } from "./bindings/GameSummary";
import { ImportPgnResult } from "./bindings/ImportPgnResult";



export const startGame = async (
  humanColor: PieceColor,
  timeControl: TimeControl,
): Promise<GameCreateResponse> => {
  return invoke<GameCreateResponse>("start_game", { humanColor, timeControl })
    .then((response) => {
      return response;
    })
    .catch((err) => {
      console.error("start_game failed:", err);
      throw err;
    });
};

export const endGame = (
  reason: TerminationReason,
  losingSide?: PieceColor,
): Promise<GameResult> => {
  return invoke<GameResult>("end_game", { reason, losingSide });
};



export const makeMove = (
  from: Square,
  to: Square,
  promotion: PieceType | null = null,
): Promise<GameStateView> => {
  return invoke<GameStateView>("make_move", { from, to, promotion });
};

export const getGames = (): Promise<GameSummary[]> => {
  return invoke<GameSummary[]>("get_games");
};

export const deleteGame = (gameId: number): Promise<void> => {
  return invoke("delete_game", { gameId });
};

// Queues a fresh analysis pass for an already-saved game. Resolves once
// it's enqueued, not when the pass completes. `humanColor` picks the side
// to grade for an imported game that has none stored yet.
export const analyzeGame = (
  gameId: number,
  humanColor?: PieceColor,
): Promise<void> => {
  return invoke("analyze_game", { gameId, humanColor });
};

export const importPgn = (pgn: string): Promise<ImportPgnResult> => {
  return invoke<ImportPgnResult>("import_pgn", { pgn });
};
