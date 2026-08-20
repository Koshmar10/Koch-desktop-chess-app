import { useState, type ReactNode } from "react";
import { ChessboardContext } from "./ChessboardContext";
import { BOARD_SIZE, SQUARE_SIZE } from "./lib/constants";
import { STARTING_POSITION } from "./lib/startingPosition";
import type { PlacedPiece } from "./lib/types";

interface ChessboardProps {
  children: ReactNode;
}

const Chessboard: React.FC<ChessboardProps> = ({ children }) => {
  const [pieces, setPieces] = useState<PlacedPiece[]>(STARTING_POSITION);
  const [selectedSquare, setSelectedSquare] = useState<[number, number] | null>(null);

  const selectSquare = (row: number, col: number) => {
    setSelectedSquare([row, col]);
  };

  const clearSelection = () => {
    setSelectedSquare(null);
  };

  const movePiece = (from: [number, number], to: [number, number]) => {
    setPieces((prev) => {
      const withoutCaptured = prev.filter((p) => !(p.row === to[0] && p.col === to[1]));
      return withoutCaptured.map((p) =>
        p.row === from[0] && p.col === from[1] ? { ...p, row: to[0], col: to[1] } : p
      );
    });
    setSelectedSquare(null);
  };

  const boardPixelSize = SQUARE_SIZE * BOARD_SIZE;

  return (
    <ChessboardContext.Provider
      value={{
        boardSize: BOARD_SIZE,
        squareSize: SQUARE_SIZE,
        pieces,
        selectedSquare,
        selectSquare,
        movePiece,
        clearSelection,
      }}
    >
      <div
        className="relative"
        style={{ width: boardPixelSize, height: boardPixelSize }}
      >
        {children}
      </div>
    </ChessboardContext.Provider>
  );
};

export default Chessboard;
