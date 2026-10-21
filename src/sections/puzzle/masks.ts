import type { LastMove } from "../../api/bindings/LastMove";
import { TONE_SQUARE_COLOR } from "../../components/chessboard/lib/highlightTones";
import type {
  HighlightTone,
  OverlayMask,
} from "../../components/chessboard/lib/types";

// The puzzle board only ever draws one of these at a time, so there's no
// order among them to keep; the name says what the number is for.
const MOVE_RESULT_PRIORITY = 0;

/**
 * A move's two squares, filled in one tone: how the puzzle answers a
 * move, whether it was the solution's or not.
 */
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

// One builder per result, as with the analyzer's engine moves: each fixes
// its own id and colour, so no call site can mark a mistake in green.
export const wrongMoveMask = (move: LastMove): OverlayMask =>
  moveResultMask("puzzle:wrong", move, "danger");

export const correctMoveMask = (move: LastMove): OverlayMask =>
  moveResultMask("puzzle:correct", move, "good");
