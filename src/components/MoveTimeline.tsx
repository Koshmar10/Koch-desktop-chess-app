import { useEffect, useRef } from "react";
import type { MoveQuality } from "../api/bindings/MoveQuality";
import { MOVE_QUALITY_COLOR } from "../lib/moveQuality";
import EmptyNote from "./EmptyNote";

const ROW_HEIGHT_CLASS = "h-7";

/**
 * A move's grade as a coloured dot. Ungraded moves — the opponent's, or
 * any move of a game that hasn't been analysed — get an empty slot of the
 * same width rather than nothing, so the move names stay in one straight
 * line down each column.
 */
const QualityDot = ({ quality }: { quality: MoveQuality | null }) =>
  quality === null ? (
    <span className="h-2 w-2 shrink-0" />
  ) : (
    <span
      role="img"
      aria-label={quality}
      title={quality}
      className="h-2 w-2 shrink-0 rounded-full"
      style={{ backgroundColor: MOVE_QUALITY_COLOR[quality] }}
    />
  );

interface MoveCellProps {
  san: string | undefined;
  clock: string | null | undefined;
  // Undefined when the list shows no grades at all; null for a move that
  // simply has none.
  quality: MoveQuality | null | undefined;
  ply: number;
  isViewed: boolean;
  // Left out where there's nowhere to jump to — a game still being played.
  onSelect?: (ply: number) => void;
}

const MoveCell = ({
  san,
  clock,
  quality,
  ply,
  isViewed,
  onSelect,
}: MoveCellProps) => {
  const ref = useRef<HTMLButtonElement>(null);

  // Keeps the viewed move in sight however it got there — the toolbar, the
  // timeline chart, the keyboard, or a new move during play. `nearest` only
  // scrolls when the move is out of view, and only as far as it takes, so
  // clicking a move that's already visible never jumps the list.
  useEffect(() => {
    if (isViewed) ref.current?.scrollIntoView({ block: "nearest" });
  }, [isViewed]);

  // Black's column is short by one on an odd-length game. An empty cell
  // rather than nothing at all, so the two columns stay in step and the
  // numbers beside them keep lining up.
  if (san === undefined) {
    return <div className={ROW_HEIGHT_CLASS} />;
  }

  // A move you can't jump to doesn't pretend you can: no pointer cursor,
  // no hover wash.
  const interaction = onSelect ? "cursor-pointer hover:bg-primary/15" : "";

  return (
    <button
      ref={ref}
      type="button"
      onClick={onSelect && (() => onSelect(ply))}
      disabled={!onSelect}
      className={`${ROW_HEIGHT_CLASS} flex items-center justify-between gap-2 rounded px-2 text-sm font-medium transition-colors duration-100 ${
        isViewed
          ? "bg-primary/30 text-foreground"
          : `text-foreground/80 ${interaction}`
      }`}
    >
      <span className="flex items-center gap-1.5">
        {quality !== undefined && <QualityDot quality={quality} />}
        <span>{san}</span>
      </span>
      {clock && (
        <span className="text-xs tabular-nums text-foreground/35">{clock}</span>
      )}
    </button>
  );
};

interface MoveTimelineProps {
  /** SAN in ply order, White's first. */
  moves: string[];
  /**
   * Remaining clock after each ply, same indexing as `moves`. Optional:
   * a game imported without clock annotations has none, and nothing in
   * the current backend carries them yet — the column simply disappears.
   */
  clocks?: (string | null)[];
  /**
   * Each move's grade, same indexing as `moves`. Optional, and mostly
   * null even when present: only the human's moves of an analysed game
   * are graded.
   */
  qualities?: (MoveQuality | null)[];
  // The highlighted move. -1, before the first move, highlights nothing.
  viewedPly: number;
  // Clicking a move jumps to it. Left out, the list is read-only — the
  // play screen's, where the board always shows the latest move.
  onSelectPly?: (ply: number) => void;
}

/**
 * Three columns — move numbers, White, Black — the way the old app's
 * "Move Timeline" laid them out, rather than one row per pair. Keeping
 * each side in its own column means a long SAN on one side never shifts
 * the other side's moves out of alignment.
 *
 * Shared by the analyzer's Game tab and the play screen, so a game's moves
 * look the same while it's played and when it's looked back on.
 */
const MoveTimeline = ({
  moves,
  clocks,
  qualities,
  viewedPly,
  onSelectPly,
}: MoveTimelineProps) => {
  if (moves.length === 0) {
    return <EmptyNote>No moves yet</EmptyNote>;
  }

  // Grade slots only appear once there's at least one grade to show — an
  // unanalysed game shouldn't carry a column of empty space for them.
  const showsQualities = qualities?.some((q) => q !== null) ?? false;

  const moveNumbers = Array.from(
    { length: Math.ceil(moves.length / 2) },
    (_, i) => i + 1,
  );

  return (
    <div className="flex flex-row">
      <div className="flex min-w-6 flex-col items-end pr-2">
        {moveNumbers.map((number) => (
          <div
            key={number}
            className={`${ROW_HEIGHT_CLASS} flex items-center justify-end text-xs tabular-nums text-foreground/40`}
          >
            {number}.
          </div>
        ))}
      </div>

      {[0, 1].map((sideOffset) => (
        <div key={sideOffset} className="flex flex-1 flex-col">
          {moveNumbers.map((_, pairIndex) => {
            const ply = pairIndex * 2 + sideOffset;
            return (
              <MoveCell
                key={ply}
                san={moves[ply]}
                clock={clocks?.[ply]}
                quality={
                  showsQualities ? (qualities?.[ply] ?? null) : undefined
                }
                ply={ply}
                isViewed={viewedPly === ply}
                onSelect={onSelectPly}
              />
            );
          })}
        </div>
      ))}
    </div>
  );
};

export default MoveTimeline;
