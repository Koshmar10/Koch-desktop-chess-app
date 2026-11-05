import { Flag, RotateCcw } from "lucide-react";
import { OutlineButton, PrimaryButton } from "../../components/buttons";
import DialogCard, { NARROW_DIALOG_PX } from "../../components/DialogCard";
import { useGameContext } from "./GameContext";

const ICON_SIZE_PX = 16;

// No close button: the board behind it is waiting on a move that isn't
// coming, so retrying or surrendering are the only ways on.
const EngineErrorCard = () => {
  const { engineError, retryEngineReply, endGame } = useGameContext();

  return (
    <DialogCard title="Stockfish couldn't move" widthPx={NARROW_DIALOG_PX}>
      <p className="text-sm break-words text-foreground/70">{engineError}</p>
      <div className="flex items-center gap-3">
        <PrimaryButton onClick={retryEngineReply}>
          <RotateCcw size={ICON_SIZE_PX} />
          Retry
        </PrimaryButton>
        <OutlineButton onClick={() => endGame()}>
          <Flag size={ICON_SIZE_PX} />
          Surrender
        </OutlineButton>
      </div>
    </DialogCard>
  );
};

export default EngineErrorCard;
