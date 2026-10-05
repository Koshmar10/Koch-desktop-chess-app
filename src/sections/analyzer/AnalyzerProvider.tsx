import { ReactNode, useCallback, useEffect, useState } from "react";
import { sandboxMove, sandboxNew } from "../../api/analyzer";
import { GameReplay } from "../../api/bindings/GameReplay";
import { PieceColor } from "../../api/bindings/PieceColor";
import { ReplayPosition } from "../../api/bindings/ReplayPosition";
import { SandboxStarted } from "../../api/bindings/SandboxStarted";
import { Square } from "../../api/bindings/Square";
import {
  AnalyzerContext,
  GameAnalyzerContextValue,
  SandboxAnalyzerContextValue,
} from "./AnalyzerContext";
import { PanelTab } from "./panel/SidePanel";
import { START_POSITION_PLY } from "./types";
import { useGameReplay } from "./useGameReplay";

export type AnalyzerMode = "game" | "sandbox";

export interface AnalyzerProviderProps {
  children: ReactNode
}

/**
 * What the analyzer remembers across screens: the last game opened, so the
 * sidebar and the mode menu can take you back to it. The mode itself isn't
 * stored — it's the route (`/analysis` is the sandbox, `/analysis/:gameId`
 * a game), so it can never disagree with what's on screen.
 */
export function AnalyzerProvider({ children }: AnalyzerProviderProps) {
  const [loadedGameId, setLoadedGameId] = useState<number | null>(null);
  const { replay, error, isLoading } = useGameReplay(loadedGameId);
  // The game analyzer's view of the loaded game, kept here so it survives
  // leaving the screen — same starting values `Analyzer` uses today.
  const [viewedPly, setViewedPly] = useState(START_POSITION_PLY);
  const [isFlipped, setIsFlipped] = useState(false);
  const [shownMasks, setShownMasks] = useState<ReadonlySet<string>>(
    () => new Set(),
  );
  const [selectedTab, setSelectedTab] = useState<PanelTab>("game");

  const game: GameAnalyzerContextValue = {
    replay,
    error,
    isLoading,
    viewedPly,
    setViewedPly,
    isFlipped,
    setIsFlipped,
    shownMasks,
    setShownMasks,
    selectedTab,
    setSelectedTab,
  };

  // The sandbox: the backend holds the board; this mirrors what's on screen
  // from it, so it too survives leaving the analyzer.
  const [sandboxPositions, setSandboxPositions] = useState<ReplayPosition[]>(
    [],
  );
  const [sandboxPly, setSandboxPly] = useState(START_POSITION_PLY);
  const [sandboxShownMasks, setSandboxShownMasks] = useState<
    ReadonlySet<string>
  >(() => new Set());
  const [sandboxTurn, setSandboxTurn] = useState<PieceColor>("white");
  const [sandboxLegalMoves, setSandboxLegalMoves] = useState<
    SandboxStarted["legal_moves"]
  >({});

  // Started once, on the standard position, so its findings are there the
  // first time the sandbox is opened rather than after a first move.
  useEffect(() => {
    sandboxNew(null)
      .then((started) => {
        setSandboxPositions(started.positions);
        setSandboxTurn(started.turn);
        setSandboxLegalMoves(started.legal_moves);
        setSandboxPly(START_POSITION_PLY);
      })
      .catch(console.error);
  }, []);

  // Moves are only offered from the latest position, so the new one always
  // lands one ply past where the view was.
  const makeMove = useCallback(async (from: Square, to: Square) => {
    const moved = await sandboxMove(from, to, null);
    setSandboxPositions((previous) => [...previous, moved.position]);
    setSandboxTurn(moved.turn);
    setSandboxLegalMoves(moved.legal_moves);
    setSandboxPly((previous) => previous + 1);
    return moved.position;
  }, []);

  const sandbox: SandboxAnalyzerContextValue = {
    positions: sandboxPositions,
    viewedPly: sandboxPly,
    setViewedPly: setSandboxPly,
    shownMasks: sandboxShownMasks,
    setShownMasks: setSandboxShownMasks,
    turn: sandboxTurn,
    legalMoves: sandboxLegalMoves,
    makeMove,
  };

  return (
    <AnalyzerContext.Provider
      value={{ loadedGameId, setLoadedGameId, game, sandbox }}
    >
      {children}
    </AnalyzerContext.Provider>
  );
}
