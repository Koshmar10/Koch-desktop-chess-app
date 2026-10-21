import { useEffect, useState } from "react";
import { RefreshCw, TrendingUp, Trophy } from "lucide-react";
import type { Square } from "../../api/bindings/Square";
import { OutlineButton } from "../../components/buttons";
import LoadError from "../../components/LoadError";
import LoadingSpinner from "../../components/LoadingSpinner";
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
import { BOARD_SIZE_PX } from "../../components/chessboard/lib/constants";
import { opponentOf, pieceAt } from "../../components/chessboard/lib/pieces";
import { sameSquare } from "../../components/chessboard/lib/square";
import type { PlacedPiece } from "../../components/chessboard/lib/types";
import MistakeList from "./MistakeList";
import { usePuzzleContext } from "./PuzzleContext";
import SolvedCard from "./SolvedCard";
import StreakGauge from "./StreakGauge";
import { correctMoveMask, wrongMoveMask } from "./masks";
import type { WrongMove } from "./types";

const REPLY_DELAY_MS = 600;
const WRONG_MOVE_SHOWN_MS = 500;
const SOLVED_CARD_DELAY_MS = 600;
const PANEL_WIDTH_PX = 320;
const STAT_ICON_SIZE_PX = 16;
const NO_PIECES: PlacedPiece[] = [];

const Puzzle = () => {
  const {
    puzzle,
    progress,
    error,
    isLoading,
    ensureLoaded,
    newPuzzle,
    updateProgress,
    points,
    streak,
    recordSolvedPuzzle,
    breakStreak,
  } = usePuzzleContext();
  const [wrongMove, setWrongMove] = useState<WrongMove | null>(null);

  const positions = puzzle?.positions ?? [];
  const solverColor = puzzle?.solver_color ?? null;
  const currentPly = progress?.currentPly ?? 0;
  const viewedPly = progress?.viewedPly ?? 0;
  const turn = progress?.turn ?? null;
  const mistakes = progress?.mistakes ?? [];
  const solvedCard = progress?.solvedCard ?? "hidden";
  const lastPly = positions.length - 1;
  const isSolved = puzzle !== null && currentPly === lastPly;

  useEffect(() => ensureLoaded(), [ensureLoaded]);

  useEffect(() => {
    if (solverColor === null || turn === solverColor || isSolved) return;
    const timer = setTimeout(
      () =>
        updateProgress((current) => ({
          ...current,
          currentPly: current.currentPly + 1,
          viewedPly: current.currentPly + 1,
          turn: solverColor,
        })),
      REPLY_DELAY_MS,
    );
    return () => clearTimeout(timer);
  }, [solverColor, turn, currentPly, isSolved, updateProgress]);

  useEffect(() => {
    if (wrongMove === null) return;
    const timer = setTimeout(() => setWrongMove(null), WRONG_MOVE_SHOWN_MS);
    return () => clearTimeout(timer);
  }, [wrongMove]);

  useEffect(() => {
    if (!isSolved || solvedCard !== "hidden") return;
    const timer = setTimeout(
      () => updateProgress((current) => ({ ...current, solvedCard: "shown" })),
      SOLVED_CARD_DELAY_MS,
    );
    return () => clearTimeout(timer);
  }, [isSolved, solvedCard, updateProgress]);

  const canPlayerMove =
    turn !== null &&
    turn === solverColor &&
    viewedPly === currentPly &&
    wrongMove === null;
  const legalMoves =
    puzzle !== null && currentPly >= puzzle.start
      ? puzzle.legal_moves[currentPly - puzzle.start]
      : undefined;
  // Moves still to come are the solution, so only played ones are listed.
  const playedMoves = positions
    .slice(1, currentPly + 1)
    .map((position) => position.san ?? "");
  const viewedPosition = positions[viewedPly];
  const viewedMove = viewedPosition?.last_move ?? null;
  const isSolverMove =
    puzzle !== null &&
    viewedPly > puzzle.start &&
    (viewedPly - puzzle.start) % 2 === 1;
  const resultMasks = wrongMove
    ? [wrongMoveMask(wrongMove.move)]
    : isSolverMove && viewedMove
      ? [correctMoveMask(viewedMove)]
      : [];

  const goToPly = (ply: number) =>
    updateProgress((current) => ({ ...current, viewedPly: ply }));

  const handleMove = (from: Square, to: Square) => {
    if (puzzle === null) return;
    const pieces = positions[currentPly].pieces;
    const expected = positions[currentPly + 1]?.last_move;
    if (
      expected &&
      sameSquare(expected.from, from) &&
      sameSquare(expected.to, to)
    ) {
      // Scored outside the updater: Strict Mode runs updaters twice.
      const earned =
        currentPly + 1 === lastPly
          ? recordSolvedPuzzle(puzzle.rating, mistakes.length)
          : null;
      updateProgress((current) => ({
        ...current,
        currentPly: current.currentPly + 1,
        viewedPly: current.currentPly + 1,
        turn: opponentOf(puzzle.solver_color),
        earnedPoints: earned ?? current.earnedPoints,
      }));
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
    updateProgress((current) => ({
      ...current,
      mistakes: [...current.mistakes, mistake],
    }));
    setWrongMove({ move: { from, to }, pieces: applyMove(pieces, from, to) });
  };

  const handleNewPuzzle = () => {
    if (!isSolved) breakStreak();
    setWrongMove(null);
    newPuzzle();
  };

  const boardOverlay = isLoading ? (
    <LoadingSpinner message="Loading puzzle…" />
  ) : error !== null ? (
    <LoadError what="a puzzle" message={error} />
  ) : solvedCard === "shown" ? (
    <SolvedCard
      mistakes={mistakes.length}
      points={progress?.earnedPoints ?? 0}
      onNext={handleNewPuzzle}
      onClose={() =>
        updateProgress((current) => ({ ...current, solvedCard: "dismissed" }))
      }
    />
  ) : null;

  return (
    <div className="flex h-full w-full items-center justify-center">
      <div
        className="grid items-start gap-x-6 gap-y-3"
        style={{
          gridTemplateColumns: `${BOARD_SIZE_PX}px ${PANEL_WIDTH_PX}px`,
        }}
      >
        <div className="col-start-1 row-start-1 flex justify-center">
          {/* The move components count plies from -1, the start position. */}
          <MoveNavigation
            viewedPly={viewedPly - 1}
            totalPlies={currentPly}
            onFirst={() => goToPly(0)}
            onPrev={() => goToPly(viewedPly - 1)}
            onNext={() => goToPly(viewedPly + 1)}
            onLast={() => goToPly(currentPly)}
          />
        </div>

        <div className="col-start-1 row-start-2">
          <Chessboard
            key={puzzle?.id}
            pieces={wrongMove?.pieces ?? viewedPosition?.pieces ?? NO_PIECES}
            lastMove={wrongMove?.move ?? viewedMove}
            legalMoves={canPlayerMove ? legalMoves : undefined}
            flipped={solverColor === "black"}
            canPlayerMove={canPlayerMove}
            humanColor={solverColor}
            onMove={handleMove}
          >
            <Squares />
            <OverlayLayer masks={resultMasks} />
            <PieceLayer />
            {boardOverlay && <BoardOverlay>{boardOverlay}</BoardOverlay>}
          </Chessboard>
        </div>

        <div
          className="col-start-2 row-start-2 flex flex-col gap-4"
          style={{ height: BOARD_SIZE_PX }}
        >
          <PanelCard>
            <PanelSection title="Session">
              <StatRow
                icon={
                  <Trophy size={STAT_ICON_SIZE_PX} className="text-primary" />
                }
                label="Points"
                value={points}
              />
              <StreakGauge streak={streak} />
            </PanelSection>
          </PanelCard>

          <PanelCard fill>
            {puzzle === null ? (
              error !== null && (
                <OutlineButton onClick={newPuzzle}>Try again</OutlineButton>
              )
            ) : (
              <>
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
                      <TrendingUp
                        size={STAT_ICON_SIZE_PX}
                        className="text-primary"
                      />
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
                    onSelectPly={(ply) => goToPly(ply + 1)}
                  />
                </PanelSection>
              </>
            )}
          </PanelCard>
        </div>
      </div>
    </div>
  );
};

export default Puzzle;
