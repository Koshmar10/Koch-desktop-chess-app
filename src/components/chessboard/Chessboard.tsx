import { useState, type ReactNode } from "react";
import { ChessboardContext } from "./ChessboardContext";
import { applyMove } from "./lib/applyMove";
import { BOARD_PIXEL_SIZE, BOARD_SIZE, SQUARE_SIZE } from "./lib/constants";
import { STARTING_POSITION } from "./lib/startingPosition";
import type { PieceColor, PlacedPiece } from "./lib/types";
import type { GameStateView } from "../../api/bindings/GameStateView";
import type { Square } from "../../api/bindings/Square";

const toSquare = ([rank, file]: [number, number]): Square => ({ rank, file });

interface ChessboardProps {
  children: ReactNode;
  // Left undefined, the board just shows the starting position locally —
  // callers (Play, eventually Analyzer) own the real game state and pass it
  // in, so Chessboard itself never depends on any particular screen/context.
  pieces?: PlacedPiece[];
  // Show the board from Black's perspective. Defaults to false (White's
  // view) — callers derive this from whichever color the human is playing.
  flipped?: boolean;
  // Whether it's currently a legal moment to move anything at all.
  canPlayerMove?: boolean;
  // Which color the human is playing — combined with canPlayerMove to
  // decide whether a *specific* piece is draggable, not just whether
  // interaction is allowed in general. Null before a game exists.
  humanColor?: PieceColor | null;
  // Given, movePiece calls this instead of applying the move locally — the
  // real move goes wherever this sends it (e.g. the backend), and the
  // board picks up the result once `pieces` changes. Left undefined,
  // movePiece just reshuffles locally (e.g. a board with no backend
  // behind it at all).
  onMove?: (from: Square, to: Square) => void;
  // Piece id -> its legal destination squares. Left undefined, nothing
  // gets highlighted as a legal move.
  legalMoves?: GameStateView["legal_moves"];
}

const Chessboard: React.FC<ChessboardProps> = ({
  children,
  pieces: externalPieces,
  flipped = false,
  canPlayerMove = false,
  humanColor = null,
  onMove,
  legalMoves,
}) => {
  const [pieces, setPieces] = useState<PlacedPiece[]>(
    externalPieces ?? STARTING_POSITION,
  );
  // Adjusting state during render (not via useEffect+setState, which would
  // cause an extra cascading render) to reset local pieces whenever a new
  // externalPieces reference comes in — e.g. after start_game resolves.
  const [prevExternalPieces, setPrevExternalPieces] = useState(externalPieces);
  if (externalPieces !== prevExternalPieces) {
    setPrevExternalPieces(externalPieces);
    if (externalPieces) {
      setPieces(externalPieces);
    }
  }

  const [selectedSquare, setSelectedSquare] = useState<
    [number, number] | null
  >(null);

  const selectSquare = (row: number, col: number) => {
    setSelectedSquare([row, col]);
  };

  const clearSelection = () => {
    setSelectedSquare(null);
  };

  const movePiece = (from: [number, number], to: [number, number]) => {
    if (onMove) {
      onMove(toSquare(from), toSquare(to));
    } else {
      setPieces((prev) => applyMove(prev, from, to));
    }
    setSelectedSquare(null);
  };

  const selectedPiece = selectedSquare
    ? pieces.find(
        (p) =>
          p.square.rank === selectedSquare[0] &&
          p.square.file === selectedSquare[1],
      )
    : undefined;
  const legalDestinations = selectedPiece
    ? (legalMoves?.[selectedPiece.id] ?? [])
    : [];

  return (
    <ChessboardContext.Provider
      value={{
        boardSize: BOARD_SIZE,
        squareSize: SQUARE_SIZE,
        pieces,
        legalDestinations,
        isFlipped: flipped,
        selectedSquare,
        selectSquare,
        movePiece,
        clearSelection,
        canPlayerMove,
        humanColor,
      }}
    >
      <div
        className="relative"
        style={{ width: BOARD_PIXEL_SIZE, height: BOARD_PIXEL_SIZE }}
      >
        {children}
      </div>
    </ChessboardContext.Provider>
  );
};

export default Chessboard;
