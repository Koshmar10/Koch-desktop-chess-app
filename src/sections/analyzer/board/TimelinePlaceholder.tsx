import { Area, AreaChart, CartesianGrid, Line, LineChart, ReferenceDot, ReferenceLine, Tooltip, XAxis, YAxis } from "recharts";
import { BOARD_PIXEL_SIZE, BOARD_SIZE, SQUARE_SIZE } from "../../../components/chessboard/lib/constants";
import { TimelineMove } from "../Analyzer";
import { evalBarWhiteFraction, formatEvalScore } from "./evalBar";
import { Divide } from "lucide-react";

const TIMELINE_HEIGHT_PX = 152;

/**
 * Reserved space for the timeline strip — the eval curve from
 * `centipawn_history`, a move-quality dot per ply, and the move list, all
 * sharing one ply axis with the scrubber.
 *
 * Empty on purpose: it runs entirely on `GameAnalysis`, which already
 * exists, so it needs no engine session — but it does need a loaded game,
 * and there's no way to load one into this screen yet.
 */
interface TimelinePlaceholderProps {
  moves: TimelineMove[]
  // The ply on the board, marked on the curve.
  viewedPly: number
  // Clicking the chart jumps the board to the move under the cursor.
  onSelectPly: (ply: number) => void
}
const TimelinePlaceholder = ({
  moves,
  viewedPly,
  onSelectPly,
}: TimelinePlaceholderProps) => {
  const data = moves.map((move) => ({
    ...move,
    // 0 = Black winning, 0.5 = equal, 1 = White winning
    white: evalBarWhiteFraction({ kind: "cp", centipawns: move.score }),
  }));
  // Undefined before the first move, and on a move with no stored eval —
  // there's no point on the curve to mark then.
  const current = data.find((point) => point.index - 1 === viewedPly);
  return (
    <div
      className="flex shrink-0 items-center justify-center rounded-lg border-[1px] border-border/80 bg-secondary/40"
      style={{ height: TIMELINE_HEIGHT_PX }}
    >
      {/* <span className="text-xs font-semibold tracking-wide text-foreground/30 uppercase">
      Timeline
    </span> */}
      <AreaChart
        width={BOARD_PIXEL_SIZE * 0.95}
        height={TIMELINE_HEIGHT_PX * 0.9}
        data={data}
        style={{ cursor: "pointer" }}
        // The tooltip index is a position in `data`, not a ply — moves with
        // no stored eval are left out, so the two can differ. Looking the
        // point up gives its real ply. A click off any point does nothing.
        onClick={(state) => {
          const point = data[Number(state.activeTooltipIndex)];
          if (point) onSelectPly(point.index - 1);
        }}
      >
        <XAxis dataKey="index" />
        <YAxis domain={[0, 1]} hide />                      {/* fixed, so 0.5 is always the middle */}
        {/* Theme tokens from App.css, so the chart follows light and dark
            mode like everything else. */}
        <ReferenceLine
          y={0.5}
          stroke="var(--muted-foreground)"
          strokeOpacity={0.5}
          strokeDasharray="3 3"
        />
        <Area
          dataKey="white"
          baseValue={0.5}
          type="linear"
          stroke="var(--primary)"
          strokeWidth={1.5}
          fill="var(--primary)"
          fillOpacity={0.25}
          activeDot={{ r: 3, fill: "var(--primary)", stroke: "var(--background)" }}
          isAnimationActive={false}
        />
        {/* Where the board is: a faint line down the chart and a dot on the
            curve, so it reads at a glance which move you're looking at. */}
        {current && (
          <ReferenceLine
            x={current.index}
            stroke="var(--primary)"
            strokeOpacity={0.4}
          />
        )}
        {current && (
          <ReferenceDot
            x={current.index}
            y={current.white}
            r={4}
            fill="var(--primary)"
            stroke="var(--background)"
            strokeWidth={2}
          />
        )}
        <Tooltip
          cursor={{ stroke: "var(--primary)", strokeOpacity: 0.4 }}
          contentStyle={{
            backgroundColor: "var(--popover)",
            border: "1px solid var(--border)",
            borderRadius: "var(--radius)",
            color: "var(--popover-foreground)",
          }}
          labelStyle={{ color: "var(--muted-foreground)" }}
          itemStyle={{ color: "var(--primary)" }}
          // The chart plots the 0–1 bar fraction; the tooltip shows the eval
          // it came from, formatted the way the eval bar shows it.
          formatter={(_value, _name, item) => [
            formatEvalScore({ kind: "cp", centipawns: item.payload.score }),
            "Eval",
          ]}
        />
      </AreaChart>

    </div >
  )
};
export default TimelinePlaceholder;
