import { useEffect, useState } from "react";
import { TrendingUp } from "lucide-react";
import { getPuzzle } from "../../api/puzzle";
import type { ChessPuzzle } from "../../api/bindings/ChessPuzzle";
import type { PieceColor } from "../../api/bindings/PieceColor";
import type { Square } from "../../api/bindings/Square";
import LoadError from "../../components/LoadError";
import LoadingSpinner from "../../components/LoadingSpinner";
import MoveNavigation from "../../components/MoveNavigation";
import MoveTimeline from "../../components/MoveTimeline";
import PanelSection from "../../components/PanelSection";
import Chessboard from "../../components/chessboard/Chessboard";
import BoardOverlay from "../../components/chessboard/layers/BoardOverlay";
import PieceLayer from "../../components/chessboard/layers/PieceLayer";
import Squares from "../../components/chessboard/layers/Squares";
import { BOARD_SIZE_PX } from "../../components/chessboard/lib/constants";
import { opponentOf, pieceAt } from "../../components/chessboard/lib/pieces";
import { sameSquare } from "../../components/chessboard/lib/square";
import type { PlacedPiece } from "../../components/chessboard/lib/types";

// How long the opponent waits before replying, so the solver sees their own
// move land first.
const REPLY_DELAY_MS = 600;
// The analyzer's side panel width, so the two screens' panels match.
const PANEL_WIDTH_PX = 320;
const STAT_ICON_SIZE_PX = 16;
// The board until a puzzle arrives. Empty rather than the starting
// position, so the puzzle's pieces appear where they stand instead of
// every one sliding there from its starting square. One array, not a new
// `[]` each render, which the board would take for a new position.
const NO_PIECES: PlacedPiece[] = [];

const Puzzle = () => {
  const [puzzle, setPuzzle] = useState<ChessPuzzle | null>(null);
  const [error, setError] = useState<string | null>(null);
  // Both are indexes into `puzzle.positions`, 0 being the start position.
  // How far the solve has got: moves are only played from here.
  const [currentPly, setCurrentPly] = useState<number>(0);
  // What's on screen. Stepping back moves it over positions already played;
  // every new move brings it back to `currentPly`.
  const [viewedPly, setViewedPly] = useState<number>(0);
  // Whose move it is now. Starts on the opponent's side, for the move that
  // sets the puzzle up, and flips with every move played.
  const [turn, setTurn] = useState<PieceColor>("white");
  // Legal moves that weren't the solution's.
  const [mistakes, setMistakes] = useState<number>(0);

  // Every fetch is a different random puzzle, and Strict Mode runs this
  // twice in development. Only the last run's answer is used, so the board
  // doesn't load one puzzle and then slide every piece over to another.
  useEffect(() => {
    let isCurrent = true;
    getPuzzle()
      .then((data) => {
        if (!isCurrent) return;
        // DEBUG: remove before the PR.
        console.log(
          "puzzle solution:",
          data.positions
            .slice(data.start + 1)
            .map((position) => `${position.san} (${position.uci})`),
        );
        setPuzzle(data);
        // One position before the solver's, so the opponent's move the
        // puzzle answers is played in front of them, as on Lichess.
        setCurrentPly(data.start - 1);
        setViewedPly(data.start - 1);
        setTurn(opponentOf(data.turn));
      })
      .catch((err) => {
        if (isCurrent) setError(String(err));
      });
    return () => {
      isCurrent = false;
    };
  }, []);

  // The opponent's moves play themselves: whenever it's their turn and the
  // line goes on, the next position follows after a pause. The cleanup
  // drops a pending reply if the puzzle changes or the screen closes first.
  useEffect(() => {
    if (puzzle === null || turn === puzzle.turn) return;
    if (currentPly >= puzzle.positions.length - 1) return;
    const timer = setTimeout(() => {
      setCurrentPly(currentPly + 1);
      setViewedPly(currentPly + 1);
      setTurn(puzzle.turn);
    }, REPLY_DELAY_MS);
    return () => clearTimeout(timer);
  }, [puzzle, turn, currentPly]);

  // The board is drawn either way, so the screen keeps its shape; what's
  // happening shows over it, as on Play and the Analyzer.
  const isLoading = puzzle === null && error === null;
  // Only from the position the solve has reached, as in the analyzer's
  // sandbox: moving from an earlier one would branch off the line.
  const canPlayerMove =
    puzzle !== null && turn === puzzle.turn && viewedPly === currentPly;
  const legalMoves =
    puzzle !== null && currentPly >= puzzle.start
      ? puzzle.legal_moves[currentPly - puzzle.start]
      : undefined;
  // Only what's been played so far. `positions` already holds the rest of
  // the solution, and listing it would give the answer away.
  const playedMoves =
    puzzle?.positions
      .slice(1, currentPly + 1)
      .map((position) => position.san ?? "") ?? [];

  // Only the solution's move is played. An illegal drop goes back to its
  // square by itself; a legal wrong one is a mistake.
  const handleMove = (from: Square, to: Square) => {
    if (puzzle === null || puzzle?.positions[currentPly] === undefined) return;
    const expected = puzzle.positions[currentPly + 1]?.last_move;
    if (
      expected &&
      sameSquare(expected.from, from) &&
      sameSquare(expected.to, to)
    ) {
      setCurrentPly(currentPly + 1);
      setViewedPly(currentPly + 1);
      setTurn(opponentOf(puzzle.turn));
      return;
    }
    const piece = pieceAt(puzzle?.positions[currentPly].pieces, from);
    const isLegal =
      piece !== undefined &&
      (legalMoves?.[piece.id] ?? []).some((square) => sameSquare(square, to));
    if (isLegal) {
      setMistakes(mistakes + 1);
    }
  };

  return (
    <div className="flex h-full w-full flex-row items-center justify-center gap-6">
      <Chessboard
        // The board keeps a legally dropped piece on its new square until
        // the next position arrives, and a mistake brings none. A new key
        // remounts the board, which puts the piece back.
        key={mistakes}
        pieces={puzzle?.positions[viewedPly]?.pieces ?? NO_PIECES}
        lastMove={puzzle?.positions[viewedPly]?.last_move ?? null}
        legalMoves={canPlayerMove ? legalMoves : undefined}
        flipped={puzzle?.turn === "black"}
        canPlayerMove={canPlayerMove}
        humanColor={puzzle?.turn ?? null}
        onMove={handleMove}
      >
        <Squares />
        <PieceLayer />
        {isLoading && (
          <BoardOverlay>
            <LoadingSpinner message="Loading puzzle…" />
          </BoardOverlay>
        )}
        {error !== null && (
          <BoardOverlay>
            <LoadError what="a puzzle" message={error} />
          </BoardOverlay>
        )}
      </Chessboard>
      {/* The same card as the analyzer's side panel and Play's, at the
          board's height. Empty until the puzzle arrives: the board's
          overlay already says it's loading. */}
      <div
        className="flex flex-col gap-4 rounded-lg border-[1px] border-border/80 bg-secondary/40 p-3 shadow-sm"
        style={{ width: PANEL_WIDTH_PX, height: BOARD_SIZE_PX }}
      >
        <PanelSection title="Puzzle" divided>
          <div className="flex items-center gap-2 text-md">
            <TrendingUp size={STAT_ICON_SIZE_PX} className="text-primary" />
            <span className="text-foreground/50">Rating</span>
            <span className="ml-auto font-semibold text-foreground/90">
              {puzzle?.rating}
            </span>
          </div>
        </PanelSection>
        <PanelSection title="Themes" divided>
          <div className="flex flex-wrap gap-1.5">
            {puzzle?.themes.map((theme) => (
              <span
                key={theme}
                className="rounded-md border-[1px] border-border/60 bg-secondary/30 px-2 py-0.5 text-xs text-foreground/70"
              >
                {theme}
              </span>
            ))}
          </div>
        </PanelSection>
        {/* The shared move components count plies from -1, the start
            position, so a position's ply is its index minus one. */}
        <div className="flex justify-center">
          <MoveNavigation
            viewedPly={viewedPly - 1}
            totalPlies={currentPly}
            onFirst={() => setViewedPly(0)}
            onPrev={() => setViewedPly(viewedPly - 1)}
            onNext={() => setViewedPly(viewedPly + 1)}
            onLast={() => setViewedPly(currentPly)}
          />
        </div>
        <PanelSection title="Moves" fill>
          <MoveTimeline
            moves={playedMoves}
            viewedPly={viewedPly - 1}
            onSelectPly={(ply) => setViewedPly(ply + 1)}
          />
        </PanelSection>
      </div>
    </div>
  );
};

export default Puzzle;
