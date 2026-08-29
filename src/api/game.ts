import { invoke } from "@tauri-apps/api/core"
import { GameStateView } from "./bindings/GameStateView"
import { PieceType } from "./bindings/PieceType"
import { Square } from "./bindings/Square"
import { PieceColor } from "./bindings/PieceColor"
import { GameCreateResponse } from "./bindings/GameCreateResponse";
import { TerminationReason } from "./bindings/TerminationReason";
import { GameResult } from "./bindings/GameResult";
import { TimeControl } from "./bindings/TimeControl";



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
