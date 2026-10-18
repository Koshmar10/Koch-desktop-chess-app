import type { ReactNode } from "react";
import { useCallback, useEffect, useState } from "react";
import {
  sandboxLoad,
  sandboxMove,
  sandboxNew,
  sandboxUndo,
} from "../../api/analyzer";
import { analyzeGame } from "../../api/game";
import { useAppContext } from "../../app/AppContext";
import type { PieceColor } from "../../api/bindings/PieceColor";
import type { ReplayPosition } from "../../api/bindings/ReplayPosition";
import type { SandboxStarted } from "../../api/bindings/SandboxStarted";
import type { Square } from "../../api/bindings/Square";
import type {
  GameAnalyzerContextValue,
  SandboxAnalyzerContextValue,
} from "./AnalyzerContext";
import { AnalyzerContext } from "./AnalyzerContext";
import { START_POSITION_PLY, type PanelTab } from "./types";
import { useGameReplay } from "./useGameReplay";

interface AnalyzerProviderProps {
  children: ReactNode;
}

/**
 * What the analyzer remembers across screens: the last game opened, so the
 * sidebar and the mode menu can take you back to it. The mode itself isn't
 * stored — it's the route (`/analysis` is the sandbox, `/analysis/:gameId`
 * a game), so it can never disagree with what's on screen.
 */
const AnalyzerProvider = ({ children }: AnalyzerProviderProps) => {
  const [loadedGameId, setLoadedGameId] = useState<number | null>(null);
  const [isCopyOverlayOpen, setIsCopyOverlayOpen] = useState<boolean>(false);
  const [isGameSelectOverlayOpen, setIsGameSelectOverlayOpen] =
    useState<boolean>(false);
  const [isImportOverlayOpen, setIsImportOverlayOpen] =
    useState<boolean>(false);

  const { analysisStages } = useAppContext();
  const analysisStage =
    loadedGameId === null ? undefined : analysisStages[loadedGameId];
  const { replay, error, isLoading } = useGameReplay(
    loadedGameId,
    analysisStage?.kind === "Done",
  );
  // The game analyzer's view of the loaded game, kept here so it survives
  // leaving the screen — same starting values `Analyzer` uses today.
  const [viewedPly, setViewedPly] = useState(START_POSITION_PLY);
  const [isFlipped, setIsFlipped] = useState(false);
  const [shownMasks, setShownMasks] = useState<ReadonlySet<string>>(
    () => new Set(),
  );
  const [selectedTab, setSelectedTab] = useState<PanelTab>("game");

  // Opening a different game starts it fresh — first ply, nothing lit, its
  // Game tab. This state outlives the screen, so remounting the analyzer no
  // longer resets it; this has to. Reopening the same game keeps your place.
  const openGame = useCallback(
    (gameId: number) => {
      if (gameId === loadedGameId) return;
      setLoadedGameId(gameId);
      setViewedPly(START_POSITION_PLY);
      setShownMasks(new Set());
      setSelectedTab("game");
    },
    [loadedGameId],
  );

  // No side passed: a koch game always has one stored, and so does an
  // import that's been analysed before. One that hasn't is refused, and
  // the toolbar keeps the button off for it.
  const reanalyze = useCallback(async () => {
    if (loadedGameId === null) return;
    await analyzeGame(loadedGameId);
  }, [loadedGameId]);

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
    analysisStage,
    reanalyze,
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
  const [sandboxGeneration, setSandboxGeneration] = useState(0);

  // Whatever replaced the sandbox, the view lands on its last position — the
  // one moves can be made from. `positions[0]` is the start, so that
  // position's ply is two less than the count (START_POSITION_PLY when
  // there are no moves yet).
  const replaceSandbox = useCallback((started: SandboxStarted) => {
    setSandboxPositions(started.positions);
    setSandboxTurn(started.turn);
    setSandboxLegalMoves(started.legal_moves);
    setSandboxPly(started.positions.length - 2);
    setSandboxGeneration((previous) => previous + 1);
  }, []);

  // Started once, on the standard position, so its findings are there the
  // first time the sandbox is opened rather than after a first move.
  useEffect(() => {
    sandboxNew(null).then(replaceSandbox).catch(console.error);
  }, [replaceSandbox]);

  const startSandbox = useCallback(
    async (fen: string | null) => replaceSandbox(await sandboxNew(fen)),
    [replaceSandbox],
  );
  const loadSandbox = useCallback(
    async (notation: string) => replaceSandbox(await sandboxLoad(notation)),
    [replaceSandbox],
  );
  const undoSandboxMove = useCallback(
    async () => replaceSandbox(await sandboxUndo()),
    [replaceSandbox],
  );

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
    generation: sandboxGeneration,
    start: startSandbox,
    load: loadSandbox,
    undo: undoSandboxMove,
  };

  return (
    <AnalyzerContext.Provider
      value={{
        loadedGameId,
        openGame,
        game,
        sandbox,
        isCopyOverlayOpen,
        setIsCopyOverlayOpen,
        isGameSelectOverlayOpen,
        setIsGameSelectOverlayOpen,
        isImportOverlayOpen,
        setIsImportOverlayOpen,
      }}
    >
      {children}
    </AnalyzerContext.Provider>
  );
};

export default AnalyzerProvider;
