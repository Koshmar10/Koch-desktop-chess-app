import { useEffect, useState, type ReactNode } from "react";
import { listen } from "@tauri-apps/api/event";
import {
  startGame as apiStartGame,
  endGame as apiEndGame,
  makeMove as apiMakeMove,
} from "../../api/game";
import type { GameAnalysis } from "../../api/bindings/GameAnalysis";
import type { GameCreateResponse } from "../../api/bindings/GameCreateResponse";
import type { GameStateView } from "../../api/bindings/GameStateView";
import type { PieceColor } from "../../api/bindings/PieceColor";
import type { Square } from "../../api/bindings/Square";
import type { TerminationReason } from "../../api/bindings/TerminationReason";
import {
  GameContext,
  MODE_TIME_CONTROL,
  type ColorPreference,
  type GameMode,
} from "./GameContext";

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
  const [humanColor, setHumanColor] = useState<PieceColor | null>(null);
  const [colorPreference, setColorPreference] =
    useState<ColorPreference>("random");
  const [selectedMode, setSelectedMode] = useState<GameMode>("Rapid");
  const [isStartingGame, setIsStartingGame] = useState(false);

  // The backend pushes this once, well after the game-ending response
  // itself, whenever a game finishes (checkmate/stalemate via make_move,
  // or resignation/timeout via end_game) — not something either of those
  // commands returns directly.
  useEffect(() => {
    const unlisten = listen<GameAnalysis>("game-analysis-complete", (event) => {
      setAnalysis(event.payload);
      setAnalysisProgress(null);
    });
    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  useEffect(() => {
    const unlisten = listen<number>("update-analysis-progress", (event) => {
      setAnalysisProgress(event.payload);
    });
    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  // The backend pushes this once the engine's own reply has finished
  // thinking — `make_move` returns right after the human's move, it
  // doesn't wait on this.
  useEffect(() => {
    const unlisten = listen<GameStateView>("engine-move-complete", (event) => {
      setGame((prev) => (prev ? { ...prev, state: event.payload } : prev));
      setGameReceivedAt(Date.now());
    });
    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  const cycleColorPreference = () => {
    setColorPreference((prev) => nextColorPreference(prev));
  };

  const dismissResult = () => {
    setIsResultDismissed(true);
  };

  const startGame = async () => {
    setIsStartingGame(true);
    try {
      const color =
        colorPreference === "random" ? randomColor() : colorPreference;
      const response = await apiStartGame(color, MODE_TIME_CONTROL[selectedMode]);
      setHumanColor(color);
      setGame(response);
      setGameReceivedAt(Date.now());
      setAnalysis(null);
      setAnalysisProgress(null);
      setIsResultDismissed(false);
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
      const result = await apiEndGame(reason, losingSide ?? humanColor ?? undefined);
      // Keep `game` around with its terminal result rather than clearing
      // it — the result card reads `game.state.result` to know the game
      // ended, and the final position/clocks/orientation should stay on
      // screen behind it rather than snapping back to placeholders.
      setGame((prev) =>
        prev ? { ...prev, state: { ...prev.state, result } } : prev,
      );
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
