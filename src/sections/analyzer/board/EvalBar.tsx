import { BOARD_PIXEL_SIZE } from "../../../components/chessboard/lib/constants";
import { evalBarWhiteFraction, formatEvalScore } from "./evalBar";
import type { EvalScore } from "../types";

const BAR_WIDTH_PX = 34;

/**
 * Below this much fill, the label no longer has white behind it and has to
 * flip to a light colour to stay readable — the case that shows up exactly
 * when White is getting crushed, which is when the number matters most.
 */
const LABEL_ON_WHITE_MIN_FRACTION = 0.08;

interface EvalBarProps {
  // Always White-relative, whoever is to move — see `EvalScore`. Null
  // before the engine has reported anything.
  score: EvalScore | null;
  // Board shown from Black's side, so White's end of the bar is the top.
  isFlipped: boolean;
}

const EvalBar = ({ score, isFlipped }: EvalBarProps) => {
  const whiteFraction = evalBarWhiteFraction(score);
  const whiteEnd = isFlipped ? { top: 0 } : { bottom: 0 };
  const labelCls =
    whiteFraction < LABEL_ON_WHITE_MIN_FRACTION
      ? "text-neutral-100"
      : "text-neutral-900";

  return (
    <div
      className="relative shrink-0 overflow-hidden rounded-md border border-border bg-neutral-900"
      style={{ width: BAR_WIDTH_PX, height: BOARD_PIXEL_SIZE }}
    >
      <div
        className="absolute inset-x-0 bg-neutral-100 transition-[height] duration-150 ease-out"
        style={{ height: `${whiteFraction * 100}%`, ...whiteEnd }}
      />
      <span
        className={`absolute inset-x-0 px-1 text-center text-[11px] font-bold tabular-nums select-none ${labelCls}`}
        style={isFlipped ? { top: 4 } : { bottom: 4 }}
      >
        {formatEvalScore(score)}
      </span>
    </div>
  );
};

export default EvalBar;
