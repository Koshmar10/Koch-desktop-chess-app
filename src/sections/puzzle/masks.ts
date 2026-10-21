import type { LastMove } from "../../api/bindings/LastMove";
import { TONE_SQUARE_COLOR } from "../../components/chessboard/lib/highlightTones";
import type {
  HighlightTone,
  OverlayMask,
} from "../../components/chessboard/lib/types";

const MOVE_RESULT_PRIORITY = 0;

const moveResultMask = (
  id: string,
  move: LastMove,
  tone: HighlightTone,
): OverlayMask => ({
  id,
  priority: MOVE_RESULT_PRIORITY,
  squares: [move.from, move.to].map((square) => ({
    square,
    color: TONE_SQUARE_COLOR[tone],
  })),
});

export const wrongMoveMask = (move: LastMove): OverlayMask =>
  moveResultMask("puzzle:wrong", move, "danger");

export const correctMoveMask = (move: LastMove): OverlayMask =>
  moveResultMask("puzzle:correct", move, "good");
