import type { PieceColor } from "../../api/bindings/PieceColor";

// The graph's vertical range is derived per-game from the actual eval
// swing (see `displayRangeCp` below), bounded by these two — a floor so a
// quiet, balanced game doesn't zoom into noise, a ceiling so a forced mate
// (the backend clamps those to ±100,000) doesn't blow the axis up to
// something absurd and flatten every real swing in the game to a hairline.
const MIN_DISPLAY_CP = 200;
const MAX_DISPLAY_CP = 1000;
const GRAPH_VIEWBOX_WIDTH = 100;
const GRAPH_VIEWBOX_HEIGHT = 40;

const AXIS_LABEL_CLASS = "text-[13px] leading-none text-foreground/40";
const GRAPH_TITLE_CLASS =
  "text-xs font-medium text-foreground/60 uppercase tracking-wide self-start";

interface CentipawnGraphProps {
  history: number[];
  humanColor: PieceColor | null;
}

// `history` is `GameAnalysis.centipawn_history`, which the backend emits
// White-relative throughout, one point per ply including the starting
// position. Playing Black, that reads inverted — the line would dip when
// the player is winning — so it gets negated here to make "up" always mean
// "the player is better", which is what the axis labels below claim.
export const CentipawnGraph = ({ history, humanColor }: CentipawnGraphProps) => {
  if (history.length < 2) {
    return (
      <div className="h-16 flex items-center justify-center text-foreground/40 text-sm">
        Not enough moves to graph
      </div>
    );
  }

  const playerRelative =
    humanColor === "black" ? history.map((cp) => -cp) : history;
  const lastPly = playerRelative.length - 1;

  // Round up to the nearest pawn so the axis label is a clean integer, then
  // clamp between the floor and ceiling above.
  const maxAbsCp = Math.max(...playerRelative.map((cp) => Math.abs(cp)));
  const displayRangeCp = Math.min(
    MAX_DISPLAY_CP,
    Math.max(MIN_DISPLAY_CP, Math.ceil(maxAbsCp / 100) * 100),
  );
  const displayRangePawns = displayRangeCp / 100;

  const points = playerRelative.map((cp, idx) => {
    const clamped = Math.max(-displayRangeCp, Math.min(displayRangeCp, cp));
    const x = (idx / lastPly) * GRAPH_VIEWBOX_WIDTH;
    const y =
      GRAPH_VIEWBOX_HEIGHT / 2 -
      (clamped / displayRangeCp) * (GRAPH_VIEWBOX_HEIGHT / 2);
    return `${x.toFixed(2)},${y.toFixed(2)}`;
  });
  const path = points.map((p, idx) => `${idx === 0 ? "M" : "L"}${p}`).join(" ");

  return (
    <div className="flex flex-col gap-1">
      <span className={GRAPH_TITLE_CLASS}>Eval</span>

      <div className="relative h-20 pl-8">
        <span className={`absolute left-0 top-0 ${AXIS_LABEL_CLASS}`}>
          +{displayRangePawns}
        </span>
        <span
          className={`absolute left-0 top-1/2 -translate-y-1/2 ${AXIS_LABEL_CLASS}`}
        >
          0
        </span>
        <span className={`absolute left-0 bottom-0 ${AXIS_LABEL_CLASS}`}>
          -{displayRangePawns}
        </span>

        <span
          className={`absolute right-0 top-0 ${AXIS_LABEL_CLASS} pointer-events-none`}
        >
          You
        </span>
        <span
          className={`absolute right-0 bottom-0 ${AXIS_LABEL_CLASS} pointer-events-none`}
        >
          Opponent
        </span>

        <svg
          viewBox={`0 0 ${GRAPH_VIEWBOX_WIDTH} ${GRAPH_VIEWBOX_HEIGHT}`}
          preserveAspectRatio="none"
          className="w-full h-full"
        >
          <line
            x1={0}
            y1={GRAPH_VIEWBOX_HEIGHT / 2}
            x2={GRAPH_VIEWBOX_WIDTH}
            y2={GRAPH_VIEWBOX_HEIGHT / 2}
            className="stroke-foreground/20"
            strokeWidth={0.5}
            strokeDasharray="2,2"
          />
          <path
            d={path}
            fill="none"
            className="stroke-primary"
            strokeWidth={1.5}
            vectorEffect="non-scaling-stroke"
          />
        </svg>
      </div>

      <div className={`flex justify-between pl-8 ${AXIS_LABEL_CLASS}`}>
        <span>Move 1</span>
        <span>Move {lastPly}</span>
      </div>
    </div>
  );
};

interface TimeGraphProps {
  moveTimesMs: number[];
}

// Real data — `moveTimesMs` is `GameAnalysis.human_move_times_ms`, the
// human's own per-move times only (not the engine's).
export const TimeGraph = ({ moveTimesMs }: TimeGraphProps) => {
  if (moveTimesMs.length === 0) {
    return (
      <div className="h-16 flex items-center justify-center text-foreground/40 text-sm">
        Not enough moves to graph
      </div>
    );
  }

  const maxSeconds = Math.max(...moveTimesMs) / 1000;
  const lastMove = moveTimesMs.length;

  return (
    <div className="flex flex-col gap-1">
      <span className={GRAPH_TITLE_CLASS}>Time / move</span>

      <div className="relative h-20 pl-8">
        <span className={`absolute left-0 top-0 ${AXIS_LABEL_CLASS}`}>
          {maxSeconds.toFixed(1)}s
        </span>
        <span className={`absolute left-0 bottom-0 ${AXIS_LABEL_CLASS}`}>0s</span>

        <div className="h-full flex items-end gap-[3px]">
          {moveTimesMs.map((ms, idx) => (
            <div
              key={idx}
              className="flex-1 bg-primary/60 rounded-sm"
              style={{ height: `${(ms / 1000 / maxSeconds) * 100}%` }}
            />
          ))}
        </div>
      </div>

      <div className={`flex justify-between pl-8 ${AXIS_LABEL_CLASS}`}>
        <span>Move 1</span>
        <span>Move {lastMove}</span>
      </div>
    </div>
  );
};
