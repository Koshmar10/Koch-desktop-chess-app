import { useState } from "react";
import { MessageSquareText } from "lucide-react";
import Chessboard from "../../components/chessboard/Chessboard";
import Squares from "../../components/chessboard/layers/Squares";
import PieceLayer from "../../components/chessboard/layers/PieceLayer";
import ArrowLayer from "../../components/chessboard/layers/ArrowLayer";
import HighlightLayer from "../../components/chessboard/layers/HighlightLayer";
import { BOARD_PIXEL_SIZE } from "../../components/chessboard/lib/constants";
import { TONE_ARROW_COLOR } from "../../components/chessboard/lib/highlightTones";
import type {
  ArrowData,
  HighlightTone,
  SquareHighlight,
} from "../../components/chessboard/lib/types";
import { parseUciMove } from "../../components/chessboard/lib/uci";
import SidePanel from "./panel/SidePanel";
import AnalyzerToolbar from "./toolbar/AnalyzerToolbar";
import PanelPlaceholder from "./PanelPlaceholder";
import TimelinePlaceholder from "./board/TimelinePlaceholder";
import EvalBar from "./board/EvalBar";
import { topLineMove, topLineScore } from "./board/evalBar";
import { controlTints } from "./panel/position/controlTint";
import {
  MOCK_CLOCKS,
  MOCK_ENGINE_SNAPSHOT,
  MOCK_GAME,
  MOCK_MOVES,
  MOCK_PIECES,
  MOCK_POSITION_FINDINGS,
  MOCK_THREAT_MOVE,
} from "./mock";
import {
  NO_OVERLAYS_ACTIVE,
  START_POSITION_PLY,
  type ActiveOverlays,
  type EngineStatus,
  type Overlay,
} from "./types";

const CHAT_PANEL_WIDTH_PX = 320;
const PLACEHOLDER_ICON_SIZE = 28;

const clampPly = (ply: number, totalPlies: number): number =>
  Math.min(totalPlies - 1, Math.max(START_POSITION_PLY, ply));

/** A UCI move as a board arrow, or nothing if there isn't a move to draw. */
const moveArrow = (uci: string | null, tone: HighlightTone): ArrowData[] => {
  const move = uci ? parseUciMove(uci) : null;
  if (!move) return [];
  return [
    {
      from: [move.from.rank, move.from.file],
      to: [move.to.rank, move.to.file],
      color: TONE_ARROW_COLOR[tone],
      type: "engine",
    },
  ];
};

/**
 * The Analyzer screen's layout, on mock data.
 *
 * Nothing here talks to the backend yet: KOCH-10 (the live engine session)
 * doesn't exist, so the lines come from `mock.ts` and the board sits on the
 * starting position. The point of landing the shell first is that KOCH-10
 * then has a concrete contract to satisfy — see `types.ts`.
 *
 * The local state below is deliberately the *whole* set, and it stays small
 * enough to read in one go. KOCH-11 replaces it with a reducer/context; the
 * thing being avoided is the old Analyzer.tsx, which grew to 27 independent
 * `useState` calls in one component (KOCH-HANDOFF.md §8).
 */
const Analyzer = () => {
  const [viewedPly, setViewedPly] = useState(START_POSITION_PLY);
  const [isFlipped, setIsFlipped] = useState(false);
  // Whole-board overlays, as switches: what they draw is derived below on
  // every render, so the best-move arrow follows the search as it deepens
  // instead of freezing at whatever it was when you switched it on.
  const [overlays, setOverlays] = useState<ActiveOverlays>(NO_OVERLAYS_ACTIVE);
  const [engineStatus, setEngineStatus] = useState<EngineStatus>("running");
  const [isChatOpen, setIsChatOpen] = useState(true);
  // Lives here rather than in the panel because the board is the panel's
  // sibling, not its child — the Position tab decides what to light up,
  // the board draws it.
  const [highlights, setHighlights] = useState<SquareHighlight[]>([]);

  // Stopping the engine clears the lines rather than leaving stale ones on
  // screen attributed to a search that's no longer happening.
  const snapshot = engineStatus === "stopped" ? null : MOCK_ENGINE_SNAPSHOT;
  // The threat comes from a second search (KOCH-14's flipped-turn probe),
  // so it stops with the engine for the same reason.
  const threatMove = engineStatus === "stopped" ? null : MOCK_THREAT_MOVE;
  const bestMove = topLineMove(snapshot?.lines);

  const arrows = [
    ...highlights.flatMap((highlight) => highlight.arrows ?? []),
    ...(overlays.bestMove ? moveArrow(bestMove, "good") : []),
    ...(overlays.threat ? moveArrow(threatMove, "danger") : []),
  ];
  const tints = overlays.control ? controlTints(MOCK_POSITION_FINDINGS) : [];

  // Anything drawn at all, for the toolbar's clear button.
  const hasOverlays =
    highlights.length > 0 || Object.values(overlays).some(Boolean);

  const goToPly = (ply: number) =>
    setViewedPly(clampPly(ply, MOCK_MOVES.length));

  const toggleHighlight = (highlight: SquareHighlight) =>
    setHighlights((shown) =>
      shown.some((h) => h.id === highlight.id)
        ? shown.filter((h) => h.id !== highlight.id)
        : [...shown, highlight],
    );

  const toggleOverlay = (overlay: Overlay) =>
    setOverlays((prev) => ({ ...prev, [overlay]: !prev[overlay] }));

  const clearOverlays = () => {
    setHighlights([]);
    setOverlays(NO_OVERLAYS_ACTIVE);
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
              totalPlies={MOCK_MOVES.length}
              onGoToPly={goToPly}
              isFlipped={isFlipped}
              onFlip={() => setIsFlipped((flipped) => !flipped)}
              hasOverlays={hasOverlays}
              onClearOverlays={clearOverlays}
              isChatOpen={isChatOpen}
              onToggleChat={() => setIsChatOpen((open) => !open)}
            />

            <div className="col-start-1 row-start-2">
              <EvalBar
                score={topLineScore(snapshot?.lines)}
                isFlipped={isFlipped}
              />
            </div>

            {/* The position MOCK_MOVES reaches, so the board, the move
                count and the findings all describe the same chess. No
                `onMove`, so it's read-only: sandbox play needs both colors
                draggable, which `PieceLayer` gates on a single
                `humanColor` — that's KOCH-12's problem. */}
            <div className="col-start-2 row-start-2">
              <Chessboard pieces={MOCK_PIECES} flipped={isFlipped}>
                <Squares />
                {/* Arrows are derived from whatever's lit rather than
                    held separately: a finding switched off has to take
                    its arrow with it, and two lists to keep in step is
                    one more than is needed. */}
                <HighlightLayer
                  highlights={highlights}
                  arrows={arrows}
                  tints={tints}
                />
                <PieceLayer />
                <ArrowLayer />
              </Chessboard>
            </div>

            <div className="col-start-3 row-start-2">
              <SidePanel
                game={MOCK_GAME}
                moves={MOCK_MOVES}
                clocks={MOCK_CLOCKS}
                viewedPly={viewedPly}
                onSelectPly={goToPly}
                snapshot={snapshot}
                findings={MOCK_POSITION_FINDINGS}
                pieces={MOCK_PIECES}
                highlights={highlights}
                onToggleHighlight={toggleHighlight}
                overlays={overlays}
                onToggleOverlay={toggleOverlay}
                bestMove={bestMove}
                threatMove={threatMove}
                engineStatus={engineStatus}
                onToggleEngine={() =>
                  setEngineStatus((status) =>
                    status === "running" ? "stopped" : "running",
                  )
                }
              />
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

export default Analyzer;
