import {
  ChevronFirst,
  ChevronLast,
  ChevronLeft,
  ChevronRight,
} from "lucide-react";
import { START_POSITION_PLY } from "../lib/ply";
import TooltipButton from "./TooltipButton";

interface MoveNavigationProps {
  // START_POSITION_PLY (-1) means "before the first move".
  viewedPly: number;
  totalPlies: number;
  onFirst: () => void;
  onPrev: () => void;
  onNext: () => void;
  onLast: () => void;
}

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
