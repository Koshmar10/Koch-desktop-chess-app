export const BOARD_SIZE = 8;
export const SQUARE_SIZE = 86;
// Single source of truth for "how wide is the board" — anything that needs
// to size itself to match (player cards, the controls row) should derive
// from this instead of hardcoding a pixel value that happens to agree with
// it today.
export const BOARD_PIXEL_SIZE = SQUARE_SIZE * BOARD_SIZE;
