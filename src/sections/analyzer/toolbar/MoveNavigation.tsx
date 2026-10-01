import {
  ChevronFirst,
  ChevronLast,
  ChevronLeft,
  ChevronRight,
} from "lucide-react";
import { TooltipButton } from "../../../components/TooltipButton";
import { START_POSITION_PLY } from "../types";

const ICON_SIZE = 18;

interface MoveNavigationProps {
  // START_POSITION_PLY (-1) means "before the first move".
  viewedPly: number;
  totalPlies: number;
  onFirst: () => void;
  onPrev: () => void;
  onNext: () => void;
  onLast: () => void;
}

// Sits above the board rather than in the side panel: scrubbing is the
// thing you do most on this screen, so it belongs on the board's own axis
// where the eye already is, not off in a column of everything else.
const MoveNavigation = ({
  viewedPly,
  totalPlies,
  onFirst,
  onPrev,
  onNext,
  onLast,
}: MoveNavigationProps) => {
  const isAtStart = viewedPly === START_POSITION_PLY;
  const isAtEnd = viewedPly >= totalPlies - 1;

  return (
    <div className="flex w-fit flex-row items-center gap-3 px-3 py-2 shadow-sm">
      <TooltipButton
        icon={<ChevronFirst size={ICON_SIZE} />}
        tooltip="Start"
        onClick={onFirst}
        disabled={isAtStart}
      />
      <TooltipButton
        icon={<ChevronLeft size={ICON_SIZE} />}
        tooltip="Previous move"
        onClick={onPrev}
        disabled={isAtStart}
      />
      <TooltipButton
        icon={<ChevronRight size={ICON_SIZE} />}
        tooltip="Next move"
        onClick={onNext}
        disabled={isAtEnd}
      />
      <TooltipButton
        icon={<ChevronLast size={ICON_SIZE} />}
        tooltip="Latest move"
        onClick={onLast}
        disabled={isAtEnd}
      />
      {/* <span className="min-w-24 text-center text-xs tabular-nums text-foreground/50">
        {isAtStart ? "Start" : `Move ${viewedPly + 1} / ${totalPlies}`}
      </span> */}
    </div>
  );
};

export default MoveNavigation;
