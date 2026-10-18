// Squares per side — a count, so no unit suffix.
export const BOARD_SIZE = 8;
export const SQUARE_SIZE_PX = 90;
// Single source of truth for "how wide is the board" — anything that needs
// to size itself to match (player cards, the controls row) should derive
// from this instead of hardcoding a pixel value that happens to agree with
// it today.
export const BOARD_SIZE_PX = SQUARE_SIZE_PX * BOARD_SIZE;

// The board's own square colours. Shared so anything drawing a square off
// the board — a destination in a side panel, say — shows the colour that
// square really is, rather than a generic grey that could be either.
export const LIGHT_SQUARE_COLOR = "#f0d9b5";
export const DARK_SQUARE_COLOR = "#a37a58";
