import Popup from "../../components/popup/Popup";
import DialogCard, { NARROW_DIALOG_PX } from "../../components/DialogCard";
import { GhostButton } from "../../components/buttons";
import type { GameSummary } from "../../api/bindings/GameSummary";
import type { PieceColor } from "../../api/bindings/PieceColor";

interface AnalyzeSidePopupProps {
  game: GameSummary | null;
  onPick: (color: PieceColor) => void;
  onClose: () => void;
}

const SideButton = ({
  color,
  name,
  current,
  onClick,
}: {
  color: PieceColor;
  name: string;
  current: boolean;
  onClick: () => void;
}) => (
  <button
    type="button"
    onClick={onClick}
    className={`flex flex-1 items-center gap-2 rounded-md border px-3 py-2 text-sm text-foreground/90 transition-colors hover:border-primary hover:bg-primary/10 ${
      current ? "border-primary bg-primary/10" : "border-border"
    }`}
  >
    <span
      className={`h-3 w-3 shrink-0 rounded-full border border-white ${
        color === "white" ? "bg-neutral-100" : "bg-neutral-900"
      }`}
    />
    <span className="truncate">{name}</span>
  </button>
);

const AnalyzeSidePopup = ({ game, onPick, onClose }: AnalyzeSidePopupProps) => (
  <Popup open={game !== null}>
    <DialogCard title="Analyse which side?" widthPx={NARROW_DIALOG_PX}>
      <div className="flex gap-2">
        <SideButton
          color="white"
          name={game?.white_player ?? "White"}
          current={game?.human_color === "white"}
          onClick={() => onPick("white")}
        />
        <SideButton
          color="black"
          name={game?.black_player ?? "Black"}
          current={game?.human_color === "black"}
          onClick={() => onPick("black")}
        />
      </div>
      <div className="flex justify-end">
        <GhostButton onClick={onClose}>Cancel</GhostButton>
      </div>
    </DialogCard>
  </Popup>
);

export default AnalyzeSidePopup;
