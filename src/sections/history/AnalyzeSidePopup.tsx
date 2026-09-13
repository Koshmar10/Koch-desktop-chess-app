import Popup from "../../components/popup/Popup"
import { GameSummary } from "../../api/bindings/GameSummary"
import { PieceColor } from "../../api/bindings/PieceColor"

interface AnalyzeSidePopupProps {
  game: GameSummary | null
  onPick: (color: PieceColor) => void
  onClose: () => void
}

const SideButton = ({
  color,
  name,
  current,
  onClick,
}: {
  color: PieceColor
  name: string
  current: boolean
  onClick: () => void
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
)

const AnalyzeSidePopup = ({ game, onPick, onClose }: AnalyzeSidePopupProps) => (
  <Popup open={game !== null}>
    <div className="flex w-[min(360px,92vw)] flex-col gap-4 rounded-lg border border-border bg-card p-6 shadow-xl">
      <h2 className="text-lg font-medium text-foreground/90">Analyse which side?</h2>
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
      <button
        type="button"
        onClick={onClose}
        className="self-end rounded-md px-3 py-1.5 text-sm text-foreground/70 transition-colors hover:bg-border/60"
      >
        Cancel
      </button>
    </div>
  </Popup>
)

export default AnalyzeSidePopup
