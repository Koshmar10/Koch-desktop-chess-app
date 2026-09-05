import { createContext, useContext } from "react";
import type { Square } from "../../api/bindings/Square";
import type { LastMove } from "../../api/bindings/LastMove";
import type { PieceColor, PlacedPiece } from "./lib/types";

export interface ChessboardContextValue {
  boardSize: number;
  squareSize: number;
  pieces: PlacedPiece[];
  // Legal destinations for whichever piece sits on selectedSquare, or empty
  // if nothing's selected — derived once in Chessboard so every consumer
  // doesn't redo "find the piece at selectedSquare, look up its moves".
  legalDestinations: Square[];
  // Whether the board is shown from Black's perspective (rank/file reversed
  // on screen). `pieces`, `selectedSquare`, and `selectSquare`/`movePiece`
  // all stay in absolute board coordinates regardless — only the rendering
  // layers need this, to convert at the point they touch the screen grid.
  isFlipped: boolean;
  selectedSquare: [number, number] | null;
  selectSquare: (row: number, col: number) => void;
  movePiece: (from: [number, number], to: [number, number]) => void;
  clearSelection: () => void;
  // Whether it's currently a legal moment to move *anything* (the human's
  // turn). Whether a specific piece is draggable also needs `humanColor` —
  // canPlayerMove alone doesn't stop someone grabbing an opponent piece.
  canPlayerMove: boolean;
  humanColor: PieceColor | null;
  // The two squares of whichever move was played most recently, by either
  // side — distinct from `selectedSquare`, which is about the *next* move,
  // not the one that just happened.
  lastMove: LastMove | null;
}

export const ChessboardContext = createContext<ChessboardContextValue | null>(
  null,
);

export function useChessboardContext(): ChessboardContextValue {
  const ctx = useContext(ChessboardContext);
  if (!ctx) {
    throw new Error(
      "Chessboard subcomponents must be rendered inside <Chessboard>",
    );
  }
  return ctx;
}
