import { START_POSITION_PLY } from "../../types";

const ROW_HEIGHT_CLASS = "h-7";

interface MoveCellProps {
  san: string | undefined;
  clock: string | null | undefined;
  ply: number;
  isViewed: boolean;
  onSelect: (ply: number) => void;
}

const MoveCell = ({ san, clock, ply, isViewed, onSelect }: MoveCellProps) => {
  // Black's column is short by one on an odd-length game. An empty cell
  // rather than nothing at all, so the two columns stay in step and the
  // numbers beside them keep lining up.
  if (san === undefined) {
    return <div className={ROW_HEIGHT_CLASS} />;
  }

  return (
    <button
      type="button"
      onClick={() => onSelect(ply)}
      className={`${ROW_HEIGHT_CLASS} flex cursor-pointer items-center justify-between gap-2 rounded px-2 text-sm font-medium transition-colors duration-100 ${
        isViewed
          ? "bg-primary/30 text-foreground"
          : "text-foreground/80 hover:bg-primary/15"
      }`}
    >
      <span>{san}</span>
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
  viewedPly: number;
  onSelectPly: (ply: number) => void;
}

/**
 * Three columns — move numbers, White, Black — the way the old app's
 * "Move Timeline" laid them out, rather than one row per pair. Keeping
 * each side in its own column means a long SAN on one side never shifts
 * the other side's moves out of alignment.
 */
const MoveTimeline = ({
  moves,
  clocks,
  viewedPly,
  onSelectPly,
}: MoveTimelineProps) => {
  if (moves.length === 0) {
    return <p className="text-xs text-foreground/40 italic">No moves yet</p>;
  }

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
                ply={ply}
                isViewed={viewedPly === ply && viewedPly !== START_POSITION_PLY}
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
