import { useState, type ReactNode } from "react";
import {
  startGame as apiStartGame,
  endGame as apiEndGame,
  makeMove as apiMakeMove,
} from "../../api/game";
import type { GameAnalysis } from "../../api/bindings/GameAnalysis";
import type { GameCreateResponse } from "../../api/bindings/GameCreateResponse";
import type { PieceColor } from "../../api/bindings/PieceColor";
import type { Square } from "../../api/bindings/Square";
import type { TerminationReason } from "../../api/bindings/TerminationReason";
import { useAppEvent } from "../../app/useAppEvent";
import { MODE_TIME_CONTROL, type GameMode } from "../../lib/gameModes";
import { GameContext, type ColorPreference } from "./GameContext";

const randomColor = (): PieceColor => (Math.random() < 0.5 ? "white" : "black");

const CYCLE_ORDER: ColorPreference[] = ["random", "white", "black"];

const nextColorPreference = (current: ColorPreference): ColorPreference =>
  CYCLE_ORDER[(CYCLE_ORDER.indexOf(current) + 1) % CYCLE_ORDER.length];

interface GameProviderProps {
  children: ReactNode;
}

export function GameProvider({ children }: GameProviderProps) {
  const [game, setGame] = useState<GameCreateResponse | null>(null);
  const [gameReceivedAt, setGameReceivedAt] = useState<number | null>(null);
  const [analysis, setAnalysis] = useState<GameAnalysis | null>(null);
  const [analysisProgress, setAnalysisProgress] = useState<number | null>(null);
  const [isResultDismissed, setIsResultDismissed] = useState(false);
  const [isSharingPgn, setIsSharingPgn] = useState(false);
  const [humanColor, setHumanColor] = useState<PieceColor | null>(null);
  const [colorPreference, setColorPreference] =
    useState<ColorPreference>("random");
  const [selectedMode, setSelectedMode] = useState<GameMode>("Rapid");
  const [isStartingGame, setIsStartingGame] = useState(false);

  // The backend pushes this once, well after the game-ending response
  // itself, whenever a game finishes (checkmate/stalemate via make_move,
  // or resignation/timeout via end_game) — not something either of those
  // commands returns directly.
  useAppEvent("game-analysis-complete", (finished) => {
    setAnalysis(finished);
    setAnalysisProgress(null);
  });

  useAppEvent("update-analysis-progress", setAnalysisProgress);

  // The backend pushes this once the engine's own reply has finished
  // thinking — `make_move` returns right after the human's move, it
  // doesn't wait on this.
  useAppEvent("engine-move-complete", (state) => {
    setGame((prev) => (prev ? { ...prev, state } : prev));
    setGameReceivedAt(Date.now());
  });

  const cycleColorPreference = () => {
    setColorPreference((prev) => nextColorPreference(prev));
  };

  const dismissResult = () => {
    setIsResultDismissed(true);
  };

  const openSharePgn = () => {
    setIsSharingPgn(true);
  };

  const closeSharePgn = () => {
    setIsSharingPgn(false);
  };

  const startGame = async () => {
    setIsStartingGame(true);
    try {
      const color =
        colorPreference === "random" ? randomColor() : colorPreference;
      const response = await apiStartGame(
        color,
        MODE_TIME_CONTROL[selectedMode],
      );
      setHumanColor(color);
      setGame(response);
      setGameReceivedAt(Date.now());
      setAnalysis(null);
      setAnalysisProgress(null);
      setIsResultDismissed(false);
      setIsSharingPgn(false);
    } catch (err) {
      console.error("failed to start game:", err);
    } finally {
      setIsStartingGame(false);
    }
  };

  const endGame = async (
    reason: TerminationReason = "Resignation",
    losingSide?: PieceColor,
  ) => {
    try {
      const finalState = await apiEndGame(
        reason,
        losingSide ?? humanColor ?? undefined,
      );
      // Keep `game` around with its final state rather than clearing it —
      // the result card reads `game.state` to know the game ended and how
      // it went, and the final position/clocks/orientation should stay on
      // screen behind it rather than snapping back to placeholders.
      setGame((prev) => (prev ? { ...prev, state: finalState } : prev));
    } catch (err) {
      console.error("failed to end game:", err);
    }
  };

  const makeMove = async (from: Square, to: Square) => {
    try {
      const newState = await apiMakeMove(from, to, null);
      setGame((prev) => (prev ? { ...prev, state: newState } : prev));
      setGameReceivedAt(Date.now());
    } catch (err) {
      console.error("move failed:", err);
    }
  };

  return (
    <GameContext.Provider
      value={{
        game,
        gameReceivedAt,
        analysis,
        analysisProgress,
        isResultDismissed,
        dismissResult,
        isSharingPgn,
        openSharePgn,
        closeSharePgn,
        humanColor,
        colorPreference,
        cycleColorPreference,
        selectedMode,
        setSelectedMode,
        isStartingGame,
        startGame,
        endGame,
        makeMove,
      }}
    >
      {children}
    </GameContext.Provider>
  );
}
