import { createContext, useContext } from "react";
import type { GameAnalysis } from "../../api/bindings/GameAnalysis";
import type { GameCreateResponse } from "../../api/bindings/GameCreateResponse";
import type { PieceColor } from "../../api/bindings/PieceColor";
import type { Square } from "../../api/bindings/Square";
import type { TerminationReason } from "../../api/bindings/TerminationReason";
import type { TimeControl } from "../../api/bindings/TimeControl";

export type ColorPreference = PieceColor | "random";

export type GameMode = "Bullet" | "Blitz" | "Rapid" | "Classical";

export const MODE_TIME_CONTROL: Record<GameMode, TimeControl> = {
  Bullet: { initial_ms: 60_000, increment_ms: 0 },
  Blitz: { initial_ms: 180_000, increment_ms: 0 },
  Rapid: { initial_ms: 600_000, increment_ms: 0 },
  Classical: { initial_ms: 1_800_000, increment_ms: 0 },
};

export const formatClock = (ms: number): string => {
  const totalSeconds = Math.floor(ms / 1000);
  const minutes = Math.floor(totalSeconds / 60);
  const seconds = totalSeconds % 60;
  return `${minutes.toString().padStart(2, "0")}:${seconds.toString().padStart(2, "0")}`;
};

export interface GameContextValue {
  game: GameCreateResponse | null;
  // When `game` was last set, from Date.now() — the baseline the live
  // clock countdown measures elapsed time against.
  gameReceivedAt: number | null;
  // Arrives asynchronously, well after `game.state.result` turns
  // non-"Unfinished" — the backend runs a full Stockfish pass over the
  // finished game and pushes this in once it's done, it isn't part of the
  // move/end-game response itself.
  analysis: GameAnalysis | null;
  // True once the player has closed the result card — the game itself
  // isn't reset by this (see `endGame`, which keeps `game` around with its
  // terminal result), only whether the card is showing. Reset to false by
  // `startGame`.
  isResultDismissed: boolean;
  dismissResult: () => void;
  humanColor: PieceColor | null;
  colorPreference: ColorPreference;
  cycleColorPreference: () => void;
  selectedMode: GameMode;
  setSelectedMode: (mode: GameMode) => void;
  isStartingGame: boolean;
  startGame: () => Promise<void>;
  // losingSide defaults to humanColor (Resignation is always the human
  // clicking the button) but Timeout can hit either side, so callers can
  // override it explicitly.
  endGame: (reason?: TerminationReason, losingSide?: PieceColor) => Promise<void>;
  makeMove: (from: Square, to: Square) => Promise<void>;
}

export const GameContext = createContext<GameContextValue | null>(null);

export function useGameContext(): GameContextValue {
  const ctx = useContext(GameContext);
  if (!ctx) {
    throw new Error("useGameContext must be used inside <GameProvider>");
  }
  return ctx;
}
