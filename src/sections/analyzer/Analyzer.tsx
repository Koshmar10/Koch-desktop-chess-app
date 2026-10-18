import {
  useCallback,
  useEffect,
  useEffectEvent,
  useMemo,
  useState,
} from "react";
import { useNavigate, useParams } from "react-router-dom";
import { MessageSquareText } from "lucide-react";
import LoadingSpinner from "../../components/LoadingSpinner";
import Chessboard from "../../components/chessboard/Chessboard";
import Squares from "../../components/chessboard/layers/Squares";
import PieceLayer from "../../components/chessboard/layers/PieceLayer";
import ArrowLayer from "../../components/chessboard/layers/ArrowLayer";
import OverlayLayer from "../../components/chessboard/layers/OverlayLayer";
import BoardOverlay from "../../components/chessboard/layers/BoardOverlay";
import { STARTING_POSITION } from "../../components/chessboard/lib/startingPosition";
import { BOARD_SIZE_PX } from "../../components/chessboard/lib/constants";
import { stopLiveEngine, updateLiveEnginePosition } from "../../api/analyzer";
import SidePanel from "./panel/SidePanel";
import { MaskSelectionContext } from "./panel/maskSelection";
import AnalyzerToolbar from "./toolbar/AnalyzerToolbar";
import { nextMistakePly } from "./toolbar/nextMistake";
import PanelPlaceholder from "./PanelPlaceholder";
import EvalTimeline from "./board/EvalTimeline";
import EvalBar from "./board/EvalBar";
import { cpScore, topLineMove, topLineScore } from "./board/evalBar";
import { clamp } from "../../lib/math";
import {
  bestMoveMask,
  ghosted,
  positionMasks,
  threatMask,
} from "./overlays/masks";
import { useLiveEngine } from "./useLiveEngine";
import {
  START_POSITION_PLY,
  type EngineStatus,
  type GameActions,
  type MaskSelection,
  type SandboxActions,
  type TimelineMove,
} from "./types";
import type { Square } from "../../api/bindings/Square";
import { useAnalyzerContext } from "./AnalyzerContext";
import CopyPgnFenOverlay from "./board-overlays/CopyPgnFenOverlay";
import { buildSandboxPgn, buildSavedGamePgn } from "../play/buildPgn";
import ImportPgnFenOverlay from "./board-overlays/ImportPgnFenOverlay";
import GameSelectOverlay from "./board-overlays/GameSelectOverlay";

const CHAT_PANEL_WIDTH_PX = 320;
const PLACEHOLDER_ICON_SIZE_PX = 28;

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
 * Without one it's the sandbox, where you play moves from any position you
 * start or load. Either way the live engine searches the position on
 * screen.
 *
 * What outlives the screen — the loaded game, each mode's place and
 * overlays — lives in `AnalyzerProvider`; the local state below is only
 * what belongs to this visit, small enough to read in one go. The thing
 * being avoided is the old Analyzer.tsx, which grew to 27 independent
 * `useState` calls in one component (KOCH-HANDOFF.md §8).
 */
const Analyzer = ({ gameId }: { gameId: number | null }) => {
  const {
    game: saved,
    sandbox,
    openGame,
    isCopyOverlayOpen,
    isGameSelectOverlayOpen,
    setIsGameSelectOverlayOpen,
    isImportOverlayOpen,
    setIsImportOverlayOpen,
  } = useAnalyzerContext();
  const liveSnapshot = useLiveEngine();
  const navigate = useNavigate();
  const isGameMode = gameId !== null;

  // ---- The source: what's on screen, from whichever mode this is ----
  // Both modes produce the same ReplayPosition list; only a saved game has
  // a summary and accuracies, and everything that shows those already
  // copes with null. The sandbox never reads the game, and vice versa.
  const replay = isGameMode ? saved.replay : null;
  const positions = useMemo(
    () => (isGameMode ? (replay?.positions ?? []) : sandbox.positions),
    [isGameMode, replay, sandbox.positions],
  );
  const game = replay?.summary ?? null;
  const whiteAccuracy = replay?.white_accuracy ?? null;
  const blackAccuracy = replay?.black_accuracy ?? null;
  // Each mode keeps its own place and its own overlays, so nothing done in
  // one shows up in the other.
  const viewedPly = isGameMode ? saved.viewedPly : sandbox.viewedPly;
  const setViewedPly = isGameMode ? saved.setViewedPly : sandbox.setViewedPly;
  const shownMasks = isGameMode ? saved.shownMasks : sandbox.shownMasks;
  const setShownMasks = isGameMode
    ? saved.setShownMasks
    : sandbox.setShownMasks;

  const [engineStatus, setEngineStatus] = useState<EngineStatus>("running");
  const [isChatOpen, setIsChatOpen] = useState(true);
  // What's on the board, by mask id. Lives here rather than in the panel
  // because the board is the panel's sibling, not its child — the panel
  // decides what to light up, the board draws it.
  const [previewMaskId, setPreviewMaskId] = useState<string | null>(null);

  // ---- The position on screen ----
  // `positions[0]` is the start position, so the viewed ply is offset by
  // one — ply -1 (before any move) is index 0. With no positions yet (an
  // empty sandbox, a game still loading) everything below falls back to an
  // empty starting position, so nothing claims to be the game before it is.
  const position = positions[viewedPly + 1] ?? null;
  // Built once per list of positions rather than on every render — engine
  // snapshots re-render this screen several times a second, and a fresh
  // array each time would make every memoised child (the timeline chart)
  // redraw too.
  const moves = useMemo(
    () => positions.slice(1).map((p) => p.san ?? ""),
    [positions],
  );
  const uciMoves = useMemo(
    () => positions.slice(1).map((p) => p.uci ?? ""),
    [positions],
  );
  // One point per move that has both a stored eval and a grade, so an
  // unanalysed game — and the sandbox — has none.
  const timelineMoves: TimelineMove[] = useMemo(
    () =>
      positions
        .slice(1)
        .flatMap((p, index) =>
          p.eval_cp === null || p.quality === null
            ? []
            : [{ index: index + 1, score: p.eval_cp, quality: p.quality }],
        ),
    [positions],
  );
  const pieces = position?.pieces ?? STARTING_POSITION;
  const findings = position?.findings ?? null;
  const qualities = positions.slice(1).map((p) => p.quality);

  // ---- The live engine (KOCH-10) ----
  // What its search is for: this game, or this sandbox. The sandbox's
  // generation is in it, so once the sandbox is replaced or rewound the
  // search starts over (the effect below), and a late snapshot for its old
  // positions can't be shown against the new ones.
  const searchId = isGameMode
    ? `game-${gameId}`
    : `sandbox-${sandbox.generation}`;
  // Its name for a position: which search, and how far into it.
  const positionKeyAt = (ply: number) => `${searchId}:${ply}`;
  // Where the moves are played from — a sandbox can start on any position.
  const startFen = positions[0]?.fen ?? null;
  // Same function across renders, so `goToPly` below can be too. The key
  // is spelled out rather than calling `positionKeyAt`, which is a new
  // function every render and would defeat the point.
  const searchPosition = useCallback(
    (ply: number) =>
      updateLiveEnginePosition(
        `${searchId}:${ply}`,
        startFen,
        uciMoves.slice(0, ply + 1),
      ).catch(console.error),
    [searchId, startFen, uciMoves],
  );

  // The search starts on the position the screen opens on — after a
  // remount that's the remembered ply, not the start — and `goToPly` sends
  // each new one after that. An effect event, so it reads the latest ply
  // and moves without them being dependencies: as dependencies they'd
  // restart the search on every ply change and every render.
  const startSearch = useEffectEvent(() => {
    if (engineStatus === "running") searchPosition(viewedPly);
  });

  // Leaving the screen stops the search — an infinite one holds every
  // engine thread at full load — while the session stays for next time.
  useEffect(() => {
    startSearch();
    return () => {
      stopLiveEngine().catch(console.error);
    };
  }, [searchId]);

  // Stopping the engine clears the lines rather than leaving stale ones on
  // screen attributed to a search that's no longer happening.
  const isEngineShowing = engineStatus !== "stopped";
  // Only a snapshot for the position on screen counts, so one that arrives
  // late for the last position is never shown against this one.
  const liveForThisPosition =
    liveSnapshot?.position_key === positionKeyAt(viewedPly)
      ? liveSnapshot
      : null;
  const snapshot = isEngineShowing ? liveForThisPosition : null;
  // Nothing searches for the threat yet — that's KOCH-14's flipped-turn
  // probe — so there's none to show.
  const threatMove = null;
  const bestMove = topLineMove(snapshot?.pv_lines);

  // The live engine's top line when it has one for this position.
  // Otherwise a loaded game falls back to the eval its analysis pass
  // stored — null for the start position and for a game that hasn't been
  // analysed.
  const storedEval = position?.eval_cp ?? null;
  const storedScore = storedEval === null ? null : cpScore(storedEval);
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

  // Stable across renders that don't change what it uses — in particular
  // the snapshot renders — so the memoised timeline chart, which gets it
  // as `onSelectPly`, can skip them.
  const goToPly = useCallback(
    (ply: number) => {
      const next = clamp(ply, START_POSITION_PLY, moves.length - 1);
      if (next === viewedPly) return;
      if (engineStatus === "running") searchPosition(next);
      setViewedPly(next);
      // What's switched on stays on: it's ids, and the masks are rebuilt
      // for the new ply. The preview goes, though — the row it came from
      // may not exist on the new ply, and a row that unmounts under the
      // pointer never gets its pointer-leave to clear it.
      setPreviewMaskId(null);
    },
    [moves.length, viewedPly, engineStatus, searchPosition, setViewedPly],
  );

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
    if (engineStatus === "running") searchPosition(viewedPly);
  };

  const toggleEngine = () => {
    const willRun = engineStatus !== "running";
    setEngineStatus(willRun ? "running" : "stopped");
    if (willRun) searchPosition(viewedPly);
    else stopLiveEngine().catch(console.error);
  };

  // ---- Sandbox moves ----
  // Only in the sandbox, and only from its latest position: moving from an
  // earlier one would have to branch the line, which the sandbox doesn't do.
  const canMove = !isGameMode && viewedPly === moves.length - 1;
  const playSandboxMove = (from: Square, to: Square) => {
    // The new move's ply and the moves leading to it, read now: this
    // render's lists don't have the move yet.
    const ply = uciMoves.length;
    sandbox
      .makeMove(from, to)
      .then((played) => {
        setPreviewMaskId(null);
        if (engineStatus !== "running") return;
        updateLiveEnginePosition(positionKeyAt(ply), startFen, [
          ...uciMoves,
          played.uci ?? "",
        ]).catch(console.error);
      })
      .catch(console.error);
  };

  const clearOverlays = () => {
    setShownMasks(new Set());
    setPreviewMaskId(null);
  };

  // ---- Toolbar actions ----
  // Replacing or rewinding the sandbox moves the view, and the search
  // follows on its own: the sandbox's generation changes `searchId`.
  // Like a move, it drops the preview — its row may not exist any more.
  const replaceSandbox = (replace: Promise<void>) =>
    replace.then(() => setPreviewMaskId(null));

  // In a game you played, the mistakes worth jumping to are your own.
  const nextMistake = nextMistakePly(
    qualities,
    viewedPly,
    game?.human_color ?? null,
  );
  const gameActions: GameActions = {
    analysisStage: saved.analysisStage,
    onReanalyze:
      game !== null && game.human_color === null
        ? null
        : () => saved.reanalyze().catch(console.error),
    // Waits for the new sandbox before switching to it, so the old one
    // never flashes up first.
    onTryInSandbox: position
      ? () =>
          sandbox
            .start(position.fen)
            .then(() => navigate("/analysis"))
            .catch(console.error)
      : null,
    onNextMistake: nextMistake === null ? null : () => goToPly(nextMistake),
  };
  const sandboxActions: SandboxActions = {
    onReset: () => replaceSandbox(sandbox.start(null)).catch(console.error),
    onUndo:
      moves.length > 0
        ? () => replaceSandbox(sandbox.undo()).catch(console.error)
        : null,
  };
  // The overlay shows the error itself, so this one is passed on, not
  // logged; only a load that worked closes it.
  const loadIntoSandbox = (notation: string) =>
    replaceSandbox(sandbox.load(notation)).then(() =>
      setIsImportOverlayOpen(false),
    );
  // The same two steps as opening one from History. The new route remounts
  // this screen, but the overlay's open flag lives in the provider and
  // would survive that — so it's closed here too.
  const openPickedGame = (pickedId: number) => {
    openGame(pickedId);
    navigate(`/analysis/${pickedId}`);
    setIsGameSelectOverlayOpen(false);
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
              gridTemplateColumns: `auto ${BOARD_SIZE_PX}px auto`,
            }}
          >
            <AnalyzerToolbar
              viewedPly={viewedPly}
              totalPlies={moves.length}
              onGoToPly={goToPly}
              isFlipped={saved.isFlipped}
              onFlip={() => saved.setIsFlipped((flipped) => !flipped)}
              hasOverlays={hasOverlays}
              onClearOverlays={clearOverlays}
              isChatOpen={isChatOpen}
              onToggleChat={() => setIsChatOpen((open) => !open)}
              gameActions={gameActions}
              sandboxActions={sandboxActions}
            />

            <div className="col-start-1 row-start-2">
              <EvalBar score={evalScore} isFlipped={saved.isFlipped} />
            </div>

            {/* A saved game is read-only. In the sandbox, whichever side is
                to move is the one you drag — `humanColor` follows the turn —
                and only while you're on the latest position. */}
            <div className="col-start-2 row-start-2">
              <Chessboard
                pieces={pieces}
                flipped={saved.isFlipped}
                lastMove={position?.last_move ?? null}
                canPlayerMove={canMove}
                humanColor={sandbox.turn}
                legalMoves={canMove ? sandbox.legalMoves : undefined}
                onMove={playSandboxMove}
              >
                <Squares />
                <OverlayLayer masks={drawnMasks} />
                <PieceLayer />
                <ArrowLayer />
                {isGameMode && saved.isLoading && (
                  <BoardOverlay>
                    <LoadingSpinner message="Loading game…" />
                  </BoardOverlay>
                )}
                {isGameMode && saved.error && (
                  <BoardOverlay>
                    <GameLoadError message={saved.error} />
                  </BoardOverlay>
                )}
                {isCopyOverlayOpen && (
                  <BoardOverlay>
                    {/* A saved game's PGN has nothing in it until the game
                        has loaded; the sandbox's is whatever's been played. */}
                    <CopyPgnFenOverlay
                      pgn={
                        isGameMode
                          ? ((replay && buildSavedGamePgn(replay)) ?? "")
                          : buildSandboxPgn(positions)
                      }
                      fen={position?.fen ?? ""}
                    />
                  </BoardOverlay>
                )}
                {/* Only the sandbox imports — its toolbar has the button. */}
                {!isGameMode && isImportOverlayOpen && (
                  <BoardOverlay>
                    <ImportPgnFenOverlay onLoad={loadIntoSandbox} />
                  </BoardOverlay>
                )}
                {/* Either mode — the Game tab's button opens it from both. */}
                {isGameSelectOverlayOpen && (
                  <BoardOverlay>
                    <GameSelectOverlay
                      currentGameId={gameId}
                      onPick={openPickedGame}
                    />
                  </BoardOverlay>
                )}
              </Chessboard>
            </div>

            <div className="col-start-3 row-start-2">
              <MaskSelectionContext value={maskSelection}>
                <SidePanel
                  game={game}
                  moves={moves}
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
                  isGameMode={isGameMode}
                />
              </MaskSelectionContext>
            </div>

            <div className="col-start-2 row-start-3">
              <EvalTimeline
                moves={timelineMoves}
                viewedPly={viewedPly}
                onSelectPly={goToPly}
              />
            </div>
          </div>
        </div>
      </div>

      {isChatOpen && (
        <PanelPlaceholder
          title="Assistant"
          icon={<MessageSquareText size={PLACEHOLDER_ICON_SIZE_PX} />}
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
