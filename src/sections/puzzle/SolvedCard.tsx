import { ArrowRight, CircleCheck } from "lucide-react";
import { PrimaryButton } from "../../components/buttons";
import DialogCard, { NARROW_DIALOG_PX } from "../../components/DialogCard";

const RESULT_ICON_SIZE_PX = 40;
const BUTTON_ICON_SIZE_PX = 16;

interface SolvedCardProps {
  mistakes: number;
  // What the solve added to the session's points.
  points: number;
  onNext: () => void;
  // Closes the card, to look at the finished position.
  onClose: () => void;
}

/**
 * Shown over the board once the last move of the solution is played. No
 * title: "Solved" is the card's heading, as the verdict is on Play's
 * result card.
 */
const SolvedCard = ({ mistakes, points, onNext, onClose }: SolvedCardProps) => (
  <DialogCard onClose={onClose} widthPx={NARROW_DIALOG_PX}>
    <div className="flex flex-col items-center gap-2">
      <CircleCheck size={RESULT_ICON_SIZE_PX} className="text-emerald-500" />
      <h2 className="text-2xl font-bold tracking-tight text-foreground/90">
        Solved
      </h2>
      <span className="text-xl font-semibold text-emerald-500">
        +{points} points
      </span>
      <span className="text-sm text-foreground/60">
        {mistakes === 0
          ? "No mistakes"
          : `${mistakes} ${mistakes === 1 ? "mistake" : "mistakes"}`}
      </span>
    </div>
    <PrimaryButton onClick={onNext}>
      Next puzzle
      <ArrowRight size={BUTTON_ICON_SIZE_PX} />
    </PrimaryButton>
  </DialogCard>
);

export default SolvedCard;
