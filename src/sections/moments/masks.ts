import type { MomentKind } from "../../api/bindings/MomentKind";
import type { MomentSummary } from "../../api/bindings/MomentSummary";
import { TONE_SQUARE_COLOR } from "../../components/chessboard/lib/highlightTones";
import type {
  HighlightTone,
  OverlayMask,
} from "../../components/chessboard/lib/types";

// The only mask on a moment's card, so there is nothing yet to order it
// against.
const MOMENT_PRIORITY = 0;

// The hues of MOMENT_KIND_TEXT_CLASS, so a card's square and its label agree.
const MOMENT_KIND_TONE: Record<MomentKind, HighlightTone> = {
  material_chance: "warning",
  threatened_piece: "danger",
  winning_position: "good",
  losing_position: "danger",
};

/**
 * The square a moment is about, filled in its kind's tone. A winning or
 * losing position is about no one square, so it marks nothing.
 */
export const momentMask = (moment: MomentSummary): OverlayMask => ({
  id: `moment:${moment.moment_id}`,
  priority: MOMENT_PRIORITY,
  squares:
    moment.square === null
      ? []
      : [
          {
            square: moment.square,
            color: TONE_SQUARE_COLOR[MOMENT_KIND_TONE[moment.kind]],
          },
        ],
});
