import { useEffect, useState } from "react";
import { useParams } from "react-router-dom";
import { MessageSquareText } from "lucide-react";
import { LoadingSpinner } from "../../components/LoadingSpinner";
import Chessboard from "../../components/chessboard/Chessboard";
import Squares from "../../components/chessboard/layers/Squares";
import PieceLayer from "../../components/chessboard/layers/PieceLayer";
import ArrowLayer from "../../components/chessboard/layers/ArrowLayer";
import OverlayLayer from "../../components/chessboard/layers/OverlayLayer";
import BoardOverlay from "../../components/chessboard/layers/BoardOverlay";
import { STARTING_POSITION } from "../../components/chessboard/lib/startingPosition";
import { BOARD_PIXEL_SIZE } from "../../components/chessboard/lib/constants";
import { stopLiveEngine, updateLiveEnginePosition } from "../../api/analyzer";
import SidePanel from "./panel/SidePanel";
import { MaskSelectionContext } from "./panel/maskSelection";
import AnalyzerToolbar from "./toolbar/AnalyzerToolbar";
import PanelPlaceholder from "./PanelPlaceholder";
import TimelinePlaceholder from "./board/TimelinePlaceholder";
import EvalBar from "./board/EvalBar";
import { topLineMove, topLineScore } from "./board/evalBar";
import {
  bestMoveMask,
  ghosted,
  positionMasks,
  threatMask,
} from "./overlays/masks";
import { useGameReplay } from "./useGameReplay";
import {
  MOCK_CLOCKS,
  MOCK_ENGINE_SNAPSHOT,
  MOCK_GAME,
  MOCK_MOVES,
  MOCK_PIECES,
  MOCK_POSITION_FINDINGS,
  MOCK_BLACK_ACCURACY,
  MOCK_QUALITIES,
  MOCK_WHITE_ACCURACY,
  MOCK_THREAT_MOVE,
} from "./mock";
import {
  START_POSITION_PLY,
  type EngineStatus,
  type EvalScore,
  type MaskSelection,
} from "./types";

const CHAT_PANEL_WIDTH_PX = 320;
const PLACEHOLDER_ICON_SIZE = 28;

const clampPly = (ply: number, totalPlies: number): number =>
  Math.min(totalPlies - 1, Math.max(START_POSITION_PLY, ply));

const GameLoadError = ({ message }: { message: string }) => (
  <div className="max-w-sm rounded-lg border-[1px] border-destructive/50 bg-card p-4 text-sm text-foreground/80">
    Couldn't load this game: {message}
  </div>
);

/**
 * The Analyzer screen, in one of two modes.
 *
 * With a `gameId` it shows a saved game: every position, its findings and
 * its stored analysis come from one backend replay (`load_game_replay`).
 * Without one it's the sandbox, still on `mock.ts` until sandbox play
 * (KOCH-12) exists. A loaded game is searched by the live engine
 * (KOCH-10); the sandbox keeps mock engine lines, since its board is mock
 * data the engine has no moves for.
 *
 * The local state below is deliberately the *whole* set, and it stays small
 * enough to read in one go. KOCH-11 replaces it with a reducer/context; the
 * thing being avoided is the old Analyzer.tsx, which grew to 27 independent
 * `useState` calls in one component (KOCH-HANDOFF.md §8).
 */
const Analyzer = ({ gameId }: { gameId: number | null }) => {
  const { replay, error, isLoading, liveSnapshot } = useGameReplay(gameId);
  const isGameMode = gameId !== null;
  const [viewedPly, setViewedPly] = useState(START_POSITION_PLY);
  const [isFlipped, setIsFlipped] = useState(false);
  const [engineStatus, setEngineStatus] = useState<EngineStatus>("running");
  const [isChatOpen, setIsChatOpen] = useState(true);
  // What's on the board, by mask id. Lives here rather than in the panel
  // because the board is the panel's sibling, not its child — the panel
  // decides what to light up, the board draws it.
  const [shownMasks, setShownMasks] = useState<ReadonlySet<string>>(
    () => new Set(),
  );
  const [previewMaskId, setPreviewMaskId] = useState<string | null>(null);

  // ---- What's on screen: the loaded game's position, or the sandbox ----
  // `positions[0]` is the start position, so the viewed ply is offset by
  // one — ply -1 (before any move) is index 0.
  const position = replay?.positions[viewedPly + 1] ?? null;
  const moves = isGameMode
    ? (replay?.positions.slice(1).map((p) => p.san ?? "") ?? [])
    : MOCK_MOVES;
  const uciMoves = isGameMode
    ? (replay?.positions.slice(1).map((p) => p.uci ?? "") ?? [])
    : [];
  // While a game loads, an empty start position rather than the sandbox's
  // mock — nothing on screen should claim to be the game before it is.
  const pieces = isGameMode
    ? (position?.pieces ?? STARTING_POSITION)
    : MOCK_PIECES;
  const findings = isGameMode
    ? (position?.findings ?? null)
    : MOCK_POSITION_FINDINGS;
  const game = isGameMode ? (replay?.summary ?? null) : MOCK_GAME;
  // Nothing stores remaining clocks per move yet, only each move's time.
  const clocks = isGameMode ? undefined : MOCK_CLOCKS;
  const qualities = isGameMode
    ? replay?.positions.slice(1).map((p) => p.quality)
    : MOCK_QUALITIES;
  const whiteAccuracy = isGameMode
    ? (replay?.white_accuracy ?? null)
    : MOCK_WHITE_ACCURACY;
  const blackAccuracy = isGameMode
    ? (replay?.black_accuracy ?? null)
    : MOCK_BLACK_ACCURACY;

  // ---- The live engine (KOCH-10) ----
  // Its name for a position: which game, and how far into it.
  const positionKeyAt = (ply: number) => `${gameId}:${ply}`;
  const searchPosition = (ply: number) =>
    updateLiveEnginePosition(
      positionKeyAt(ply),
      uciMoves.slice(0, ply + 1),
    ).catch(console.error);

  // The search starts on the position the screen opens on; after that
  // `goToPly` sends each new one. Leaving the screen stops it — an infinite
  // search holds every engine thread at full load — while the session
  // stays for next time.
  useEffect(() => {
    if (!isGameMode) return;
    updateLiveEnginePosition(`${gameId}:${START_POSITION_PLY}`, []).catch(
      console.error,
    );
    return () => {
      stopLiveEngine().catch(console.error);
    };
  }, [gameId, isGameMode]);

  // Stopping the engine clears the lines rather than leaving stale ones on
  // screen attributed to a search that's no longer happening.
  const isEngineShowing = engineStatus !== "stopped";
  // Only a snapshot for the position on screen counts, so one that arrives
  // late for the last position is never shown against this one.
  const liveForThisPosition =
    liveSnapshot?.position_key === positionKeyAt(viewedPly)
      ? liveSnapshot
      : null;
  const shownSnapshot = isGameMode ? liveForThisPosition : MOCK_ENGINE_SNAPSHOT;
  const snapshot = isEngineShowing ? shownSnapshot : null;
  // The threat is still mock data — the search behind it is KOCH-14 — so a
  // loaded game shows none rather than a made-up one.
  const threatMove = isEngineShowing && !isGameMode ? MOCK_THREAT_MOVE : null;
  const bestMove = topLineMove(snapshot?.pv_lines);

  // The live engine's top line when it has one for this position.
  // Otherwise a loaded game falls back to the eval its analysis pass
  // stored — null for the start position and for a game that hasn't been
  // analysed.
  const storedEval = position?.eval_cp ?? null;
  const storedScore: EvalScore | null =
    storedEval === null ? null : { kind: "cp", centipawns: storedEval };
  const evalScore = topLineScore(snapshot?.pv_lines) ?? storedScore;

  // ---- What's drawn: everything switched on, plus the preview ----
  // Every mask this ply could draw, rebuilt each render — so an id that's
  // switched on follows the game, drawn on whichever plies its finding
  // exists on. A preview of something already switched on adds nothing;
  // anything else previewed is drawn ghosted, so "I'm looking at this"
  // stays visibly different from "I left this on".
  const drawnMasks = [
    ...positionMasks(findings, pieces),
    bestMoveMask(bestMove),
    threatMask(threatMove),
  ].flatMap((mask) => {
    if (shownMasks.has(mask.id)) return [mask];
    return mask.id === previewMaskId ? [ghosted(mask)] : [];
  });

  // Anything switched on, for the toolbar's clear button — including ids
  // with nothing to draw on this ply, which would otherwise be "on" with
  // no way to clear them from here.
  const hasOverlays = shownMasks.size > 0;

  const goToPly = (ply: number) => {
    const next = clampPly(ply, moves.length);
    if (next === viewedPly) return;
    if (isGameMode && engineStatus === "running") searchPosition(next);
    setViewedPly(next);
    // What's switched on stays on: it's ids, and the masks are rebuilt
    // for the new ply. The preview goes, though — the row it came from
    // may not exist on the new ply, and a row that unmounts under the
    // pointer never gets its pointer-leave to clear it.
    setPreviewMaskId(null);
  };

  // A click ends the preview it happened over: switching something on
  // makes the preview redundant, and switching it off has to take it off
  // the board now — not once the pointer happens to leave the row.
  const toggleMask = (id: string) => {
    setShownMasks((shown) =>
      shown.has(id)
        ? new Set([...shown].filter((shownId) => shownId !== id))
        : new Set([...shown, id]),
    );
    setPreviewMaskId(null);
  };

  // New engine settings stop the search to apply; pick it back up on the
  // position on screen, unless the engine is switched off.
  const resumeSearch = () => {
    if (isGameMode && engineStatus === "running") searchPosition(viewedPly);
  };

  const toggleEngine = () => {
    const willRun = engineStatus !== "running";
    setEngineStatus(willRun ? "running" : "stopped");
    if (!isGameMode) return;
    if (willRun) searchPosition(viewedPly);
    else stopLiveEngine().catch(console.error);
  };

  const clearOverlays = () => {
    setShownMasks(new Set());
    setPreviewMaskId(null);
  };

  const maskSelection: MaskSelection = {
    shown: shownMasks,
    previewId: previewMaskId,
    onToggle: toggleMask,
    onPreview: setPreviewMaskId,
  };

  return (
    <div className="flex h-full w-full flex-row">
      <div className="flex min-w-0 flex-1 flex-col">
        <div className="flex flex-1 items-center justify-center px-6 pb-6">
          {/* A grid rather than nested flex rows: the navigation row and
              the timeline both need to line up with the *board* column
              specifically, while the eval bar and side panel line up with
              the board's own top and bottom. Flex can satisfy one of those
              or the other, never both — one middle track pinned to the
              board's width gives all three for free. */}
          <div
            className="grid items-start gap-x-6 gap-y-3"
            style={{
              gridTemplateColumns: `auto ${BOARD_PIXEL_SIZE}px auto`,
            }}
          >
            <AnalyzerToolbar
              viewedPly={viewedPly}
              totalPlies={moves.length}
              onGoToPly={goToPly}
              isFlipped={isFlipped}
              onFlip={() => setIsFlipped((flipped) => !flipped)}
              hasOverlays={hasOverlays}
              onClearOverlays={clearOverlays}
              isChatOpen={isChatOpen}
              onToggleChat={() => setIsChatOpen((open) => !open)}
            />

            <div className="col-start-1 row-start-2">
              <EvalBar score={evalScore} isFlipped={isFlipped} />
            </div>

            {/* No `onMove`, so it's read-only: sandbox play needs both
                colors draggable, which `PieceLayer` gates on a single
                `humanColor` — that's KOCH-12's problem. */}
            <div className="col-start-2 row-start-2">
              <Chessboard
                pieces={pieces}
                flipped={isFlipped}
                lastMove={position?.last_move ?? null}
              >
                <Squares />
                <OverlayLayer masks={drawnMasks} />
                <PieceLayer />
                <ArrowLayer />
                {isLoading && (
                  <BoardOverlay>
                    <LoadingSpinner message="Loading game…" />
                  </BoardOverlay>
                )}
                {error && (
                  <BoardOverlay>
                    <GameLoadError message={error} />
                  </BoardOverlay>
                )}
              </Chessboard>
            </div>

            <div className="col-start-3 row-start-2">
              <MaskSelectionContext value={maskSelection}>
                <SidePanel
                  // A loaded game opens on its Game tab even though the
                  // summary hasn't arrived yet on the first render.
                  initialTab={isGameMode ? "game" : undefined}
                  game={game}
                  moves={moves}
                  clocks={clocks}
                  qualities={qualities}
                  whiteAccuracy={whiteAccuracy}
                  blackAccuracy={blackAccuracy}
                  viewedPly={viewedPly}
                  onSelectPly={goToPly}
                  snapshot={snapshot}
                  findings={findings}
                  pieces={pieces}
                  bestMove={bestMove}
                  threatMove={threatMove}
                  engineStatus={engineStatus}
                  onToggleEngine={toggleEngine}
                  onEngineSettingsApplied={resumeSearch}
                />
              </MaskSelectionContext>
            </div>

            <div className="col-start-2 row-start-3">
              <TimelinePlaceholder />
            </div>
          </div>
        </div>
      </div>

      {isChatOpen && (
        <PanelPlaceholder
          title="Assistant"
          icon={<MessageSquareText size={PLACEHOLDER_ICON_SIZE} />}
          blockedOn="KOCH-16 — blocked on the AI layer (KOCH-19)"
          widthPx={CHAT_PANEL_WIDTH_PX}
          borderSide="left"
        />
      )}
    </div>
  );
};

/**
 * `/analysis` is the sandbox, `/analysis/:gameId` a saved game. Keyed by
 * game, so opening a different one starts clean — back at the first ply,
 * nothing left lit from the last game — instead of each piece of state
 * needing its own reset.
 */
const AnalyzerRoute = () => {
  const { gameId } = useParams();
  const parsed = gameId === undefined ? null : Number(gameId);
  const id = parsed !== null && Number.isInteger(parsed) ? parsed : null;
  return <Analyzer key={id ?? "sandbox"} gameId={id} />;
};

export default AnalyzerRoute;
