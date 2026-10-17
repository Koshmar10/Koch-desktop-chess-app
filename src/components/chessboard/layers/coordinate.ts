import type { Square } from "../../../api/bindings/Square";
import { BOARD_SIZE } from "../lib/constants";
import { fromScreen } from "../lib/orientation";

// Raw pixel position (MouseEvent.clientX/clientY) — distinct from a board
// `Square` and a `ScreenCell` (see `lib/orientation.ts`), which are grid
// positions, not pixels.
export interface Coordinate {
  x: number;
  y: number;
}

export const coordinateFromMouseEvent = (
  e: MouseEvent | React.MouseEvent,
): Coordinate => ({ x: e.clientX, y: e.clientY });

/**
 * The board square under a pixel position, or null when it's off the
 * board. `boardOrigin` is the board's top-left corner in the same pixel
 * space — its bounding rect. Shared by the two layers that hit-test the
 * pointer: dragging pieces and drawing right-click arrows.
 */
export const squareAtPoint = (
  point: Coordinate,
  boardOrigin: { left: number; top: number },
  squareSize: number,
  isFlipped: boolean,
): Square | null => {
  const col = Math.floor((point.x - boardOrigin.left) / squareSize);
  const row = Math.floor((point.y - boardOrigin.top) / squareSize);
  const isOnBoard =
    row >= 0 && row < BOARD_SIZE && col >= 0 && col < BOARD_SIZE;
  return isOnBoard ? fromScreen([row, col], isFlipped) : null;
};
