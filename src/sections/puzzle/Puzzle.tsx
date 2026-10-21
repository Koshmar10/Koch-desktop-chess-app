import { useEffect } from "react";
import { OutlineButton } from "../../components/buttons";
import LoadError from "../../components/LoadError";
import LoadingSpinner from "../../components/LoadingSpinner";
import MoveNavigation from "../../components/MoveNavigation";
import PanelCard from "../../components/PanelCard";
import Chessboard from "../../components/chessboard/Chessboard";
import BoardOverlay from "../../components/chessboard/layers/BoardOverlay";
import Squares from "../../components/chessboard/layers/Squares";
import type { PlacedPiece } from "../../components/chessboard/lib/types";
import { START_POSITION_PLY } from "../../lib/ply";
import { usePuzzleContext } from "./PuzzleContext";
import PuzzleLayout from "./PuzzleLayout";
import PuzzleSolver from "./PuzzleSolver";

// The board until the first puzzle arrives. One array, not a new `[]` each
// render, which the board would take for a new position.
const NO_PIECES: PlacedPiece[] = [];
// The navigation's buttons before there's a puzzle. It disables them all
// itself, at the start with no plies, so these never run.
const NOTHING_TO_STEP_TO = () => {};

/**
 * The puzzle screen. The puzzle and its progress live in `PuzzleProvider`,
 * so coming back to the screen resumes where it was left; this shows them,
 * and hands them to `PuzzleSolver`.
 */
const Puzzle = () => {
  const { puzzle, solve, error, isLoading, ensureLoaded, newPuzzle } =
    usePuzzleContext();

  // Nothing is fetched until the screen first opens. After that the
  // provider already has a puzzle, and this asks for nothing.
  useEffect(() => ensureLoaded(), [ensureLoaded]);

  // What's happening, over the board: a puzzle on its way, or one that
  // didn't come.
  const boardOverlay = isLoading ? (
    <BoardOverlay>
      <LoadingSpinner message="Loading puzzle…" />
    </BoardOverlay>
  ) : error !== null ? (
    <BoardOverlay>
      <LoadError what="a puzzle" message={error} />
    </BoardOverlay>
  ) : null;

  // Before the first puzzle the toolbar, board and panel are already in
  // place, empty, so nothing moves when it arrives. The navigation is
  // there with nothing to step through, so the board doesn't drop when it
  // appears. The board is empty rather than the starting position, so the
  // puzzle's pieces appear where they stand instead of sliding there from
  // their starting squares.
  if (puzzle === null || solve === null) {
    return (
      <PuzzleLayout
        toolbar={
          <MoveNavigation
            viewedPly={START_POSITION_PLY}
            totalPlies={0}
            onFirst={NOTHING_TO_STEP_TO}
            onPrev={NOTHING_TO_STEP_TO}
            onNext={NOTHING_TO_STEP_TO}
            onLast={NOTHING_TO_STEP_TO}
          />
        }
        board={
          <Chessboard pieces={NO_PIECES}>
            <Squares />
            {boardOverlay}
          </Chessboard>
        }
        panel={
          <PanelCard fill>
            {error !== null && (
              <OutlineButton onClick={newPuzzle}>Try again</OutlineButton>
            )}
          </PanelCard>
        }
      />
    );
  }

  return (
    // Keyed by puzzle for what the solver keeps itself, like the position
    // on screen: that starts over with each puzzle, while the progress in
    // `solve` carries on.
    <PuzzleSolver
      key={puzzle.id}
      puzzle={puzzle}
      solve={solve}
      boardOverlay={boardOverlay}
    />
  );
};

export default Puzzle;
