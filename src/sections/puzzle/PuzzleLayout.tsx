import type { ReactNode } from "react";
import { BOARD_SIZE_PX } from "../../components/chessboard/lib/constants";

// The analyzer's side panel width, so the two screens' panels match.
const PANEL_WIDTH_PX = 320;

interface PuzzleLayoutProps {
  // Above the board: the move navigation.
  toolbar: ReactNode;
  board: ReactNode;
  // The side panel's cards, stacked in a column as tall as the board.
  panel: ReactNode;
}

/**
 * The toolbar over the board, and the side panel beside it. Shared by the
 * screen waiting for its first puzzle and the screen solving one, so
 * nothing moves when the puzzle arrives.
 *
 * A grid rather than nested flex rows, as on the analyzer: the toolbar
 * has to line up with the board column, while the panel lines up with the
 * board's own top and bottom. One column pinned to the board's width gives
 * both.
 */
const PuzzleLayout = ({ toolbar, board, panel }: PuzzleLayoutProps) => (
  <div className="flex h-full w-full items-center justify-center">
    <div
      className="grid items-start gap-x-6 gap-y-3"
      style={{ gridTemplateColumns: `${BOARD_SIZE_PX}px ${PANEL_WIDTH_PX}px` }}
    >
      <div className="col-start-1 row-start-1 flex justify-center">
        {toolbar}
      </div>
      <div className="col-start-1 row-start-2">{board}</div>
      <div
        className="col-start-2 row-start-2 flex flex-col gap-4"
        style={{ height: BOARD_SIZE_PX }}
      >
        {panel}
      </div>
    </div>
  </div>
);

export default PuzzleLayout;
