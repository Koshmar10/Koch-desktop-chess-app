import { DRAGGED_PIECE_Z_INDEX } from "../lib/zIndex";

// A fraction of the square, not pixels, so a piece fills the same share of
// its square on any board size — about 8px on the full-size board.
const PIECE_PADDING = 0.09;

export interface PieceVisualState {
  className: string;
  style: React.CSSProperties;
}

export const getPieceVisualState = (
  isBeingDragged: boolean,
  dragPosition: { x: number; y: number } | null,
  row: number,
  col: number,
  squareSize: number,
  squareRelativePct = 1.1,
): PieceVisualState => {
  if (isBeingDragged && dragPosition) {
    return {
      // `fixed`, so the piece in your hand can leave the board without being
      // clipped by the scrolling content area around it. No transform
      // transition here: it has to stay glued to the cursor, not trail it.
      className:
        "fixed box-border select-none pointer-events-none transition-[width,height] duration-100 ease-out",
      style: {
        zIndex: DRAGGED_PIECE_Z_INDEX,
        left: dragPosition.x - squareSize / 2,
        top: dragPosition.y - squareSize / 2,
        width: squareSize * squareRelativePct,
        height: squareSize * squareRelativePct,
        padding: squareSize * PIECE_PADDING,
      },
    };
  }

  return {
    // Placed with a transform rather than a grid cell. A grid cell can only
    // jump from one square to the next, while a transform can be
    // interpolated between them, so a move slides — and the browser animates
    // a transform on the GPU, without re-laying-out the board every frame.
    // `will-change-transform` was tried against a WebKitGTK 2.54 / Wayland
    // slide flicker; it isn't shown to help.
    className:
      "absolute top-0 left-0 box-border select-none cursor-grab transition-transform duration-200 ease-in-out will-change-transform motion-reduce:transition-none",
    style: {
      transform: `translate(${col * squareSize}px, ${row * squareSize}px)`,
      width: squareSize,
      height: squareSize,
      padding: squareSize * PIECE_PADDING,
    },
  };
};
