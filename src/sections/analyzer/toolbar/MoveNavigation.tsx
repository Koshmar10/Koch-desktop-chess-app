import {
  ChevronFirst,
  ChevronLast,
  ChevronLeft,
  ChevronRight,
} from "lucide-react";
import { TooltipButton } from "../../../components/TooltipButton";
import { START_POSITION_PLY } from "../types";

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
    <div className="flex w-fit flex-row items-center gap-2 py-2 shadow-sm">
      <TooltipButton
        icon={ChevronFirst}
        tooltip="Start"
        onClick={onFirst}
        disabled={isAtStart}
      />
      <TooltipButton
        icon={ChevronLeft}
        tooltip="Previous move"
        onClick={onPrev}
        disabled={isAtStart}
      />
      <div className="text-primary text-lg border-b-[2px] border-primary/60">
        {viewedPly + 1}
      </div>
      <TooltipButton
        icon={ChevronRight}
        tooltip="Next move"
        onClick={onNext}
        disabled={isAtEnd}
      />
      <TooltipButton
        icon={ChevronLast}
        tooltip="Latest move"
        onClick={onLast}
        disabled={isAtEnd}
      />
    </div>
  );
};

export default MoveNavigation;
