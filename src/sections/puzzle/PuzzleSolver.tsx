import { useEffect, useState, type ReactNode } from "react";
import { RefreshCw, TrendingUp, Trophy } from "lucide-react";
import type { ChessPuzzle } from "../../api/bindings/ChessPuzzle";
import type { Square } from "../../api/bindings/Square";
import MoveNavigation from "../../components/MoveNavigation";
import MoveTimeline from "../../components/MoveTimeline";
import PanelCard from "../../components/PanelCard";
import PanelSection from "../../components/PanelSection";
import StatRow from "../../components/StatRow";
import TooltipButton from "../../components/TooltipButton";
import Chessboard from "../../components/chessboard/Chessboard";
import BoardOverlay from "../../components/chessboard/layers/BoardOverlay";
import OverlayLayer from "../../components/chessboard/layers/OverlayLayer";
import PieceLayer from "../../components/chessboard/layers/PieceLayer";
import Squares from "../../components/chessboard/layers/Squares";
import { applyMove } from "../../components/chessboard/lib/applyMove";
import { opponentOf, pieceAt } from "../../components/chessboard/lib/pieces";
import { sameSquare } from "../../components/chessboard/lib/square";
import MistakeList from "./MistakeList";
import { usePuzzleContext } from "./PuzzleContext";
import PuzzleLayout from "./PuzzleLayout";
import SolvedCard from "./SolvedCard";
import StreakGauge from "./StreakGauge";
import { correctMoveMask, wrongMoveMask } from "./masks";
import type { Solve, WrongMove } from "./types";

// How long the opponent waits before replying, so the solver sees their own
// move land first.
const REPLY_DELAY_MS = 600;
// How long a wrong move stays on the board, marked red, before it's taken
// back.
const WRONG_MOVE_SHOWN_MS = 500;
// How long the finished position shows before the solved card goes over
// it, so the last move is seen landing.
const SOLVED_CARD_DELAY_MS = 600;
const STAT_ICON_SIZE_PX = 16;

interface PuzzleSolverProps {
  puzzle: ChessPuzzle;
  // How far the solve has got. Kept by the provider, so it carries on
  // where it was left when the screen opens again.
  solve: Solve;
  // Drawn over the board in place of the solved card: the next puzzle
  // loading, or failing to.
  boardOverlay: ReactNode;
}

/**
 * One puzzle being solved: the board, the moves so far and the mistakes.
 *
 * The progress comes in as `solve` and goes out through `updateSolve`.
 * What's kept here is only about what's on screen right now: the
 * position being looked at, a wrong move on its way back, the solved
 * card. The screen keys this by puzzle, so all of that starts over with a
 * new one.
 */
const PuzzleSolver = ({ puzzle, solve, boardOverlay }: PuzzleSolverProps) => {
  const {
    isLoading,
    newPuzzle,
    updateSolve,
    points,
    streak,
    recordSolve,
    breakStreak,
  } = usePuzzleContext();
  const { currentPly, turn, mistakes, earnedPoints } = solve;
  // An index into `puzzle.positions`, like `currentPly`. Stepping back
  // moves it over positions already played; every new move brings it back
  // to `currentPly`. It opens on `currentPly` too: a new puzzle's start,
  // or wherever the solve was left.
  const [viewedPly, setViewedPly] = useState(currentPly);
  // The wrong move on the board right now, until it's taken back.
  const [wrongMove, setWrongMove] = useState<WrongMove | null>(null);
  const [isSolvedCardShown, setIsSolvedCardShown] = useState(false);

  // The solution's last move has been played.
  const isSolved = currentPly === puzzle.positions.length - 1;

  // The opponent's moves play themselves: whenever it's their turn and the
  // line goes on, the next position follows after a pause. The cleanup
  // drops a pending reply if the screen closes first; coming back plays it.
  useEffect(() => {
    if (turn === puzzle.turn || isSolved) return;
    const timer = setTimeout(() => {
      updateSolve((current) => ({
        ...current,
        currentPly: current.currentPly + 1,
        turn: puzzle.turn,
      }));
      setViewedPly(currentPly + 1);
    }, REPLY_DELAY_MS);
    return () => clearTimeout(timer);
  }, [puzzle, turn, currentPly, isSolved, updateSolve]);

  // A wrong move is taken back after a moment. Dropping it hands the board
  // the position from before it again, so the piece slides back.
  useEffect(() => {
    if (wrongMove === null) return;
    const timer = setTimeout(() => setWrongMove(null), WRONG_MOVE_SHOWN_MS);
    return () => clearTimeout(timer);
  }, [wrongMove]);

  // Runs once, when the puzzle is solved. Closing the card afterwards
  // leaves it closed: `isSolved` doesn't change again.
  useEffect(() => {
    if (!isSolved) return;
    const timer = setTimeout(
      () => setIsSolvedCardShown(true),
      SOLVED_CARD_DELAY_MS,
    );
    return () => clearTimeout(timer);
  }, [isSolved]);

  // Only from the position the solve has reached, as in the analyzer's
  // sandbox: moving from an earlier one would branch off the line. Not
  // while a wrong move is still on the board, either.
  const canPlayerMove =
    turn === puzzle.turn && viewedPly === currentPly && wrongMove === null;
  const legalMoves =
    currentPly >= puzzle.start
      ? puzzle.legal_moves[currentPly - puzzle.start]
      : undefined;
  // Only what's been played so far. `positions` already holds the rest of
  // the solution, and listing it would give the answer away.
  const playedMoves = puzzle.positions
    .slice(1, currentPly + 1)
    .map((position) => position.san ?? "");
  // The solver moves first and the two sides alternate, so the move that
  // led to a position was theirs when it's an odd number of plies past the
  // start. Every move of theirs on the board is a right one: a wrong one is
  // never played.
  const isSolverMove =
    viewedPly > puzzle.start && (viewedPly - puzzle.start) % 2 === 1;
  const viewedMove = puzzle.positions[viewedPly].last_move;
  const resultMasks = wrongMove
    ? [wrongMoveMask(wrongMove.move)]
    : isSolverMove && viewedMove
      ? [correctMoveMask(viewedMove)]
      : [];

  // Only the solution's move is played. An illegal drop goes back to its
  // square by itself; a legal wrong one is a mistake, shown and then taken
  // back.
  const handleMove = (from: Square, to: Square) => {
    const pieces = puzzle.positions[currentPly].pieces;
    const expected = puzzle.positions[currentPly + 1]?.last_move;
    if (
      expected &&
      sameSquare(expected.from, from) &&
      sameSquare(expected.to, to)
    ) {
      // Scored here, as the move is played, not in an effect watching
      // `isSolved`. The score is caused by this move, and an effect would
      // also have to depend on `recordSolve`, which is a new function each
      // time the points change, so it would run again and score the same
      // puzzle twice. And outside the update below: React may run an
      // update twice in development to check it's pure, which would score
      // it twice too. Lichess solutions always end on the solver's move,
      // so the last move is always one of these.
      const isLastMove = currentPly + 1 === puzzle.positions.length - 1;
      const earned = isLastMove
        ? recordSolve(puzzle.rating, mistakes.length)
        : earnedPoints;
      updateSolve((current) => ({
        ...current,
        currentPly: current.currentPly + 1,
        turn: opponentOf(puzzle.turn),
        earnedPoints: earned,
      }));
      setViewedPly(currentPly + 1);
      return;
    }

    const piece = pieceAt(pieces, from);
    if (piece === undefined) return;
    const isLegal = (legalMoves?.[piece.id] ?? []).some((square) =>
      sameSquare(square, to),
    );
    if (!isLegal) return;

    breakStreak();
    const mistake = {
      piece,
      move: { from, to },
      isCapture: pieceAt(pieces, to) !== undefined,
    };
    updateSolve((current) => ({
      ...current,
      mistakes: [...current.mistakes, mistake],
    }));
    // Worked out here rather than asked of the backend: it's on screen for
    // half a second, never played. `applyMove` only moves the piece and
    // takes a capture, so a wrong castle, en passant or promotion shows a
    // little off for that half second.
    setWrongMove({ move: { from, to }, pieces: applyMove(pieces, from, to) });
  };

  // Leaving a puzzle unsolved for a new one is a skip, and ends the streak.
  const handleNewPuzzle = () => {
    if (!isSolved) breakStreak();
    newPuzzle();
  };

  const board = (
    <Chessboard
      pieces={wrongMove?.pieces ?? puzzle.positions[viewedPly].pieces}
      lastMove={wrongMove?.move ?? viewedMove}
      legalMoves={canPlayerMove ? legalMoves : undefined}
      flipped={puzzle.turn === "black"}
      canPlayerMove={canPlayerMove}
      humanColor={puzzle.turn}
      onMove={handleMove}
    >
      <Squares />
      <OverlayLayer masks={resultMasks} />
      <PieceLayer />
      {/* A new puzzle loading takes over from the solved card: it's what
          the card's button asked for. */}
      {boardOverlay ??
        (isSolvedCardShown && (
          <BoardOverlay>
            <SolvedCard
              mistakes={mistakes.length}
              points={earnedPoints}
              onNext={handleNewPuzzle}
              onClose={() => setIsSolvedCardShown(false)}
            />
          </BoardOverlay>
        ))}
    </Chessboard>
  );

  // The shared move components count plies from -1, the start position,
  // so a position's ply is its index minus one.
  const toolbar = (
    <MoveNavigation
      viewedPly={viewedPly - 1}
      totalPlies={currentPly}
      onFirst={() => setViewedPly(0)}
      onPrev={() => setViewedPly(viewedPly - 1)}
      onNext={() => setViewedPly(viewedPly + 1)}
      onLast={() => setViewedPly(currentPly)}
    />
  );

  const panel = (
    <>
      {/* The session's own card: it outlives this puzzle, so it stays
          apart from everything about the one on the board. */}
      <PanelCard>
        <PanelSection title="Session">
          <StatRow
            icon={<Trophy size={STAT_ICON_SIZE_PX} className="text-primary" />}
            label="Points"
            value={points}
          />
          <StreakGauge streak={streak} />
        </PanelSection>
      </PanelCard>
      <PanelCard fill>
        <PanelSection
          title="Puzzle"
          divided
          action={
            <TooltipButton
              icon={RefreshCw}
              tooltip="New puzzle"
              onClick={handleNewPuzzle}
              disabled={isLoading}
            />
          }
        >
          <StatRow
            icon={
              <TrendingUp size={STAT_ICON_SIZE_PX} className="text-primary" />
            }
            label="Rating"
            value={puzzle.rating}
          />
        </PanelSection>
        <PanelSection title="Themes" divided>
          <div className="flex flex-wrap gap-1.5">
            {puzzle.themes.map((theme) => (
              <span
                key={theme}
                className="rounded-md border-[1px] border-border/60 bg-secondary/30 px-2 py-0.5 text-xs text-foreground/70"
              >
                {theme}
              </span>
            ))}
          </div>
        </PanelSection>
        <PanelSection title={`Mistakes (${mistakes.length})`} divided>
          <MistakeList mistakes={mistakes} />
        </PanelSection>
        <PanelSection title="Moves" fill>
          <MoveTimeline
            moves={playedMoves}
            viewedPly={viewedPly - 1}
            onSelectPly={(ply) => setViewedPly(ply + 1)}
          />
        </PanelSection>
      </PanelCard>
    </>
  );

  return <PuzzleLayout toolbar={toolbar} board={board} panel={panel} />;
};

export default PuzzleSolver;
