import type { MomentDetail } from "../../api/bindings/MomentDetail";
import type { MomentSummary } from "../../api/bindings/MomentSummary";
import { X } from "lucide-react";
import { useEffect, useState, type ReactNode } from "react";
import { getMoment } from "../../api/moments";
import Chessboard from "../../components/chessboard/Chessboard";
import OverlayLayer from "../../components/chessboard/layers/OverlayLayer";
import PieceLayer from "../../components/chessboard/layers/PieceLayer";
import Squares from "../../components/chessboard/layers/Squares";
import LoadError from "../../components/LoadError";
import LoadingSpinner from "../../components/LoadingSpinner";
import MoveNavigation from "../../components/MoveNavigation";
import TooltipButton from "../../components/TooltipButton";
import { getScoreCardStyle } from "../../lib/gameSummary";
import {
  HANDLED_DOT_CLASS,
  MOMENT_KIND_ICON,
  MOMENT_KIND_LABEL,
  MOMENT_OUTCOME_LABEL,
  UNHANDLED_DOT_CLASS,
  formatPlayedDate,
} from "../../lib/moments";
import { MOVE_QUALITY_COLOR } from "../../lib/moveQuality";
import { START_POSITION_PLY } from "../../lib/ply";
import { cpScore, formatEvalScore } from "../analyzer/board/evalBar";
import { bestMoveMask } from "../analyzer/overlays/masks";
import { momentMask } from "./masks";

const BOARD_SIZE_PX = 420;
const ICON_SIZE_PX = 16;
const PLIES_PER_MOVE = 2;

const isPositionKind = (moment: MomentSummary): boolean =>
  moment.kind === "winning_position" || moment.kind === "losing_position";

// A move's moment opens on the position the move was played from; a
// position's on the one its move led to, which is what it is about.
const openingStep = (detail: MomentDetail): number =>
  Math.min(isPositionKind(detail.summary) ? 1 : 0, detail.steps.length - 1);

// The step's eval from the player's side, as the card prints it.
const playerEval = (whiteCp: number | null, moment: MomentSummary): string =>
  whiteCp === null
    ? ""
    : formatEvalScore(
        cpScore(moment.human_color === "white" ? whiteCp : -whiteCp),
      );

const DetailRow = ({ label, value }: { label: string; value: ReactNode }) => (
  <div className="flex items-baseline justify-between gap-4 text-sm">
    <span className="text-foreground/50">{label}</span>
    <span className="text-right text-foreground/90">{value}</span>
  </div>
);

interface MomentOverlayProps {
  moment: MomentSummary;
  onClose: () => void;
}

const MomentOverlay = ({ moment, onClose }: MomentOverlayProps) => {
  const [detail, setDetail] = useState<MomentDetail | null>(null);
  const [error, setError] = useState<string | null>(null);
  // Null until the user steps, so the opening step is read off the detail.
  const [chosenStep, setChosenStep] = useState<number | null>(null);

  useEffect(() => {
    let stale = false;
    getMoment(moment.moment_id)
      .then((found) => {
        if (stale) return;
        if (found === null) {
          setError("This moment is gone: its game was scanned again.");
        } else {
          setDetail(found);
        }
      })
      .catch((err) => {
        if (!stale) setError(String(err));
      });
    return () => {
      stale = true;
    };
  }, [moment.moment_id]);

  const step = detail === null ? 0 : (chosenStep ?? openingStep(detail));
  const lastStep = detail === null ? 0 : detail.steps.length - 1;
  const goToStep = (index: number) =>
    setChosenStep(Math.max(0, Math.min(index, lastStep)));

  useEffect(() => {
    const handleKey = (event: KeyboardEvent) => {
      if (event.key === "Escape") onClose();
      if (event.key === "ArrowLeft") goToStep(step - 1);
      if (event.key === "ArrowRight") goToStep(step + 1);
    };
    window.addEventListener("keydown", handleKey);
    return () => window.removeEventListener("keydown", handleKey);
  });

  const KindIcon = MOMENT_KIND_ICON[moment.kind];

  const body = () => {
    if (error !== null) {
      return <LoadError what="this moment" message={error} />;
    }
    if (detail === null) {
      return <LoadingSpinner message="Loading moment…" />;
    }
    const shown = detail.steps[step];
    const masks = [
      // The square is where the piece stood before the player's move, so
      // it is only true of the first position.
      ...(step === 0 ? [momentMask(moment)] : []),
      bestMoveMask(shown.best_move_uci),
    ];

    return (
      <div
        className="flex max-h-full gap-6 overflow-auto rounded-lg bg-card p-5 shadow-xl ring-1 ring-border"
        onClick={(event) => event.stopPropagation()}
      >
        <div className="flex flex-col items-center gap-2">
          <Chessboard
            pieces={shown.pieces}
            lastMove={shown.last_move}
            sizePx={BOARD_SIZE_PX}
            flipped={moment.human_color === "black"}
          >
            <Squares />
            <OverlayLayer masks={masks} />
            <PieceLayer />
          </Chessboard>
          <MoveNavigation
            viewedPly={step + START_POSITION_PLY}
            totalPlies={lastStep}
            onFirst={() => goToStep(0)}
            onPrev={() => goToStep(step - 1)}
            onNext={() => goToStep(step + 1)}
            onLast={() => goToStep(lastStep)}
          />
        </div>

        <div className="flex w-72 flex-col gap-4">
          <div className="flex items-start justify-between gap-2">
            <div className="flex flex-col gap-1">
              <span className="flex items-center gap-2 text-lg font-semibold">
                <KindIcon size={ICON_SIZE_PX} className="text-primary" />
                {MOMENT_KIND_LABEL[moment.kind]}
              </span>
              <span className="flex items-center gap-2 text-sm text-foreground/70">
                <span
                  className={`size-2 rounded-full ${moment.handled ? HANDLED_DOT_CLASS : UNHANDLED_DOT_CLASS}`}
                />
                {MOMENT_OUTCOME_LABEL[moment.outcome]}
              </span>
            </div>
            <TooltipButton icon={X} tooltip="Close" onClick={onClose} />
          </div>

          <div className="flex flex-col gap-1.5">
            <DetailRow
              label="Game"
              value={
                <span className="flex items-center gap-2">
                  vs {moment.opponent_name}
                  {moment.human_score !== null && (
                    <span
                      className={`rounded px-1.5 text-xs ${getScoreCardStyle(moment.human_score)}`}
                    >
                      {moment.human_score}
                    </span>
                  )}
                </span>
              }
            />
            <DetailRow
              label="Played"
              value={formatPlayedDate(moment.date_played)}
            />
            {moment.opening_name !== null && (
              <DetailRow label="Opening" value={moment.opening_name} />
            )}
            <DetailRow
              label="Move"
              value={`${Math.ceil(moment.start_ply / PLIES_PER_MOVE)} · ${moment.phase}`}
            />
            <DetailRow label="You played" value={detail.played.san} />
            {detail.best !== null && (
              <DetailRow label="Best move" value={detail.best.san} />
            )}
          </div>

          <div className="flex flex-wrap gap-1.5">
            {detail.steps.slice(1).map((moveStep, index) => (
              <button
                key={index}
                type="button"
                onClick={() => goToStep(index + 1)}
                className={`flex cursor-pointer items-center gap-1.5 rounded-md px-2 py-1 font-mono text-sm ring-1 transition-colors ${
                  step === index + 1
                    ? "bg-primary/40 ring-primary"
                    : "ring-border hover:bg-primary/20"
                }`}
              >
                {moveStep.quality !== null && (
                  <span
                    className="size-2 rounded-full"
                    style={{
                      backgroundColor: MOVE_QUALITY_COLOR[moveStep.quality],
                    }}
                  />
                )}
                {moveStep.san}
                <span className="text-xs text-foreground/50">
                  {playerEval(moveStep.eval_cp, moment)}
                </span>
              </button>
            ))}
          </div>
        </div>
      </div>
    );
  };

  return (
    // Clicking the blurred backdrop closes it; clicks inside the card stop
    // before they get here.
    <div
      className="absolute inset-0 z-100 flex items-center justify-center bg-black/30 p-6 backdrop-blur-sm"
      onClick={onClose}
    >
      {body()}
    </div>
  );
};

export default MomentOverlay;
