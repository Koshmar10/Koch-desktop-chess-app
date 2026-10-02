// The board's stacking order, in one place — every layer that raises
// itself above the others, bottom to top.
//
// The base layers carry no z-index at all: `Squares`, `OverlayLayer`'s
// fills, `PieceLayer` and `ArrowLayer` stack in the order `<Chessboard>`
// mounts them. Keep it that way for `PieceLayer` especially — a z-index on
// it would make it a stacking context, and the piece being dragged, which
// is positioned inside it, could then never rise above anything outside
// it.
//
// Applied as inline `zIndex`, not Tailwind `z-*` classes: Tailwind only
// generates classes it finds written out in the source, so a class built
// from one of these numbers at runtime would silently never exist.

/** Legal-move dots and capture rings — over the piece a ring circles. */
export const LEGAL_MOVE_MARKER_Z_INDEX = 2;

/** Finding and engine arrows — over the pieces they run between. */
export const OVERLAY_ARROWS_Z_INDEX = 10;

/** The piece in your hand — over everything drawn on the board. */
export const DRAGGED_PIECE_Z_INDEX = 50;

/** Loading and result cards, which cover the whole board. */
export const BOARD_OVERLAY_Z_INDEX = 60;
