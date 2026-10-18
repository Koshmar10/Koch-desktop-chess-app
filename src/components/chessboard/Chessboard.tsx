import { useState, type ReactNode } from "react";
import { ChessboardContext } from "./ChessboardContext";
import { applyMove } from "./lib/applyMove";
import { BOARD_SIZE, BOARD_SIZE_PX, SQUARE_SIZE_PX } from "./lib/constants";
import { pieceAt } from "./lib/pieces";
import { STARTING_POSITION } from "./lib/startingPosition";
import type { PieceColor, PlacedPiece } from "./lib/types";
import type { GameStateView } from "../../api/bindings/GameStateView";
import type { LastMove } from "../../api/bindings/LastMove";
import type { Square } from "../../api/bindings/Square";

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
  // The two squares of whichever move was played most recently. Left
  // undefined/null, nothing gets highlighted.
  lastMove?: LastMove | null;
}

const Chessboard = ({
  children,
  pieces: externalPieces,
  flipped = false,
  canPlayerMove = false,
  humanColor = null,
  onMove,
  legalMoves,
  lastMove = null,
}: ChessboardProps) => {
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

  const [selectedSquare, setSelectedSquare] = useState<Square | null>(null);

  const clearSelection = () => {
    setSelectedSquare(null);
  };

  const movePiece = (from: Square, to: Square) => {
    if (onMove) {
      onMove(from, to);
    } else {
      setPieces((prev) => applyMove(prev, from, to));
    }
    setSelectedSquare(null);
  };

  const selectedPiece = selectedSquare
    ? pieceAt(pieces, selectedSquare)
    : undefined;
  const legalDestinations = selectedPiece
    ? (legalMoves?.[selectedPiece.id] ?? [])
    : [];

  return (
    <ChessboardContext.Provider
      value={{
        boardSize: BOARD_SIZE,
        squareSize: SQUARE_SIZE_PX,
        pieces,
        legalDestinations,
        isFlipped: flipped,
        selectedSquare,
        selectSquare: setSelectedSquare,
        movePiece,
        clearSelection,
        canPlayerMove,
        humanColor,
        lastMove,
      }}
    >
      <div
        className="relative"
        style={{ width: BOARD_SIZE_PX, height: BOARD_SIZE_PX }}
      >
        {children}
      </div>
    </ChessboardContext.Provider>
  );
};

export default Chessboard;
