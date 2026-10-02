import { Flame, Lightbulb, MoveRight, type LucideIcon } from "lucide-react";
import type { Square } from "../../../../api/bindings/Square";
import type { PlacedPiece } from "../../../../components/chessboard/lib/types";
import { parseUciMove } from "../../../../components/chessboard/lib/uci";
import { maskId } from "../../overlays/masks";
import type { MaskSelection } from "../../types";
import EyeIndicator from "../EyeIndicator";
import InteractiveRow from "../InteractiveRow";
import PanelSection from "../PanelSection";
import { IconLabel, PieceChip, TileChip } from "../chips";
import {
  HOVER_ROW_CLASS,
  maskControls,
  type RowControls,
} from "../rowControls";

interface MoveRowProps {
  icon: LucideIcon;
  title: string;
  // UCI, e.g. "c6a5". Null until the engine has said anything.
  move: string | null;
  pieces: PlacedPiece[];
  controls: RowControls;
}

/**
 * One engine move, drawn the way a pin is: the piece that moves, an
 * arrow, and the square it lands on — then the eye that puts it on the
 * board.
 */
const MoveRow = ({ icon, title, move, pieces, controls }: MoveRowProps) => {
  const squares = move ? parseUciMove(move) : null;
  const pieceAt = (square: Square) =>
    pieces.find(
      (p) => p.square.rank === square.rank && p.square.file === square.file,
    );
  const mover = squares ? pieceAt(squares.from) : undefined;

  return (
    // Bottom padding is room for the square names hanging under the tiles.
    <InteractiveRow
      controls={controls}
      className={`flex items-center gap-2 pt-1 pb-4 ${HOVER_ROW_CLASS}`}
    >
      <IconLabel icon={icon} title={title} />
      {squares && mover ? (
        <>
          <PieceChip id={mover.id} pieces={pieces} />
          <MoveRight size={18} className="shrink-0 text-foreground/25" />
          {/* The bare square for a quiet move, or the piece about to be
              taken on it for a capture. */}
          <TileChip square={squares.to} piece={pieceAt(squares.to)} />
        </>
      ) : (
        // Switching it on still works with nothing to show: the arrow
        // appears as soon as the engine reports one, rather than the row
        // refusing until then.
        <span className="text-xs text-foreground/35 italic">
          waiting for the engine
        </span>
      )}
      <span className="ml-auto">
        <EyeIndicator state={controls.state} />
      </span>
    </InteractiveRow>
  );
};

interface EngineOverlaysProps {
  bestMove: string | null;
  threatMove: string | null;
  pieces: PlacedPiece[];
  maskSelection: MaskSelection;
}

/**
 * The two engine moves worth seeing on the board: what to play, and what
 * the opponent would do if you passed. Both read off live engine output,
 * which is why they live here and not in the Position tab — stop the
 * engine and they have nothing to say.
 */
const EngineOverlays = ({
  bestMove,
  threatMove,
  pieces,
  maskSelection,
}: EngineOverlaysProps) => (
  <PanelSection title="Show on board" divided>
    <MoveRow
      icon={Lightbulb}
      title="Best move"
      move={bestMove}
      pieces={pieces}
      controls={maskControls(maskId.bestMove, "the best move", maskSelection)}
    />
    <MoveRow
      icon={Flame}
      title="Main threat"
      move={threatMove}
      pieces={pieces}
      controls={maskControls(maskId.threat, "the main threat", maskSelection)}
    />
  </PanelSection>
);

export default EngineOverlays;
