import { createContext, useContext } from "react";
import type { PlacedPiece } from "./lib/types";

export interface ChessboardContextValue {
  boardSize: number;
  squareSize: number;
  pieces: PlacedPiece[];
  selectedSquare: [number, number] | null;
  selectSquare: (row: number, col: number) => void;
  movePiece: (from: [number, number], to: [number, number]) => void;
  clearSelection: () => void;
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
