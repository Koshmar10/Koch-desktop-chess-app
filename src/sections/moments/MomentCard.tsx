import type { ReactNode } from "react";
import type { MomentSummary } from "../../api/bindings/MomentSummary";
import Chessboard from "../../components/chessboard/Chessboard";
import Squares from "../../components/chessboard/layers/Squares";
import PieceLayer from "../../components/chessboard/layers/PieceLayer";
import OverlayLayer from "../../components/chessboard/layers/OverlayLayer";
import {
  HANDLED_DOT_CLASS,
  MOMENT_KIND_ICON,
  MOMENT_KIND_LABEL,
  MOMENT_OUTCOME_LABEL,
  UNHANDLED_DOT_CLASS,
  formatPlayedDate,
} from "../../lib/moments";
import { cpScore, formatEvalScore } from "../analyzer/board/evalBar";
import { momentMask } from "./masks";

export const CARD_PADDING_PX = 8;
const ICON_SIZE_PX = 14;
const PLIES_PER_MOVE = 2;

// The eval going into the moment and coming out of it. Both are already
// from the player's side, so they print as stored. A moment with only
// the first (a winning or losing position) shows just that one.
const evalSwing = ({
  eval_before_cp,
  eval_after_cp,
}: MomentSummary): string | null => {
  if (eval_before_cp === null) return null;
  const before = formatEvalScore(cpScore(eval_before_cp));
  if (eval_after_cp === null) return before;
  return `${before} → ${formatEvalScore(cpScore(eval_after_cp))}`;
};

// A marker in an icon-wide slot, so both lines' text starts at the same
// edge, then the label, then a detail pushed to the right.
const CardLine = ({
  marker,
  label,
  detail,
  className,
}: {
  marker: ReactNode;
  label: ReactNode;
  detail: ReactNode;
  className: string;
}) => (
  <div className={`flex items-center justify-between gap-2 ${className}`}>
    <span className="flex min-w-0 items-center gap-1.5">
      <span
        className="flex shrink-0 justify-center"
        style={{ width: ICON_SIZE_PX }}
      >
        {marker}
      </span>
      <span className="truncate">{label}</span>
    </span>
    <span className="shrink-0 font-mono text-[11px] text-foreground/40">
      {detail}
    </span>
  </div>
);

interface MomentCardProps {
  moment: MomentSummary;
  boardSizePx: number;
  onOpen?: () => void;
}

const MomentCard = ({ moment, boardSizePx, onOpen }: MomentCardProps) => {
  const KindIcon = MOMENT_KIND_ICON[moment.kind];

  return (
    // A ring, not a border: it takes no room, so the card is exactly the
    // board plus its padding and the board sits centred in a column sized
    // to that.
    <div
      className="rounded-md bg-card/40 ring-1 ring-border/60 hover:bg-primary/20 transition-color duration-150 ease-in cursor-pointer"
      style={{ padding: CARD_PADDING_PX }}
      onClick={onOpen}
    >
      <div className="flex items-center justify-between gap-2 px-0.5 pb-2 text-xs">
        <span className="truncate text-foreground/70">
          vs {moment.opponent_name}
        </span>
        <span className="shrink-0 font-mono text-[11px] text-foreground/40">
          {formatPlayedDate(moment.date_played)}
        </span>
      </div>
      <div className="flex items-center justify-center">
        <Chessboard
          pieces={moment.pieces}
          sizePx={boardSizePx}
          flipped={moment.human_color === "black"}
        >
          <Squares rankLabels={false} fileLabels={false} />
          <OverlayLayer masks={[momentMask(moment)]} />
          <PieceLayer squareRelativePct={200} />
        </Chessboard>
      </div>
      <div className="flex flex-col gap-1 px-0.5 pt-2.5">
        <CardLine
          className="text-sm text-foreground"
          marker={<KindIcon size={ICON_SIZE_PX} className="text-primary" />}
          label={MOMENT_KIND_LABEL[moment.kind]}
          detail={`Move ${Math.ceil(moment.start_ply / PLIES_PER_MOVE)}`}
        />
        <CardLine
          className="text-xs text-foreground/70"
          marker={
            <span
              className={`size-1.5 rounded-full ${moment.handled ? HANDLED_DOT_CLASS : UNHANDLED_DOT_CLASS}`}
            />
          }
          label={MOMENT_OUTCOME_LABEL[moment.outcome]}
          detail={
            <span className="text-foreground/70">{evalSwing(moment)}</span>
          }
        />
      </div>
    </div>
  );
};

export default MomentCard;
