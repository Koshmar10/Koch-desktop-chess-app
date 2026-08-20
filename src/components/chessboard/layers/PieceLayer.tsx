import { useCallback, useEffect, useRef, useState } from "react";
import { useChessboardContext } from "../ChessboardContext";
import { PIECE_IMAGES } from "../lib/pieceImages";
import { getPieceVisualState } from "./pieceVisualState";

// Below this drag distance, a mousedown+mouseup on a piece is treated as a click (select) rather than a drag (move).
const CLICK_THRESHOLD_PX = 6;

const PieceLayer = () => {
  const {
    pieces,
    squareSize,
    boardSize,
    selectedSquare,
    selectSquare,
    movePiece,
  } = useChessboardContext();

  const containerRef = useRef<HTMLDivElement>(null);
  const dragStartPixelRef = useRef<{ x: number; y: number } | null>(null);
  const [dragFrom, setDragFrom] = useState<[number, number] | null>(null);
  const [dragPosition, setDragPosition] = useState<{
    x: number;
    y: number;
  } | null>(null);

  const squareFromPoint = useCallback(
    (clientX: number, clientY: number): [number, number] | null => {
      const bounds = containerRef.current?.getBoundingClientRect();
      if (!bounds) return null;
      const col = Math.floor((clientX - bounds.left) / squareSize);
      const row = Math.floor((clientY - bounds.top) / squareSize);
      if (row < 0 || row >= boardSize || col < 0 || col >= boardSize)
        return null;
      return [row, col];
    },
    [boardSize, squareSize],
  );

  useEffect(() => {
    if (!dragFrom) return;

    const handleMouseMove = (e: MouseEvent) => {
      setDragPosition({ x: e.clientX, y: e.clientY });
    };

    const handleMouseUp = (e: MouseEvent) => {
      const startPixel = dragStartPixelRef.current;
      const to = squareFromPoint(e.clientX, e.clientY);
      const movedDistance = startPixel
        ? Math.hypot(e.clientX - startPixel.x, e.clientY - startPixel.y)
        : Infinity;

      if (movedDistance < CLICK_THRESHOLD_PX) {
        selectSquare(dragFrom[0], dragFrom[1]);
      } else if (to && (to[0] !== dragFrom[0] || to[1] !== dragFrom[1])) {
        movePiece(dragFrom, to);
      }

      dragStartPixelRef.current = null;
      setDragPosition(null);
      setDragFrom(null);
    };

    window.addEventListener("mousemove", handleMouseMove);
    window.addEventListener("mouseup", handleMouseUp);
    return () => {
      window.removeEventListener("mousemove", handleMouseMove);
      window.removeEventListener("mouseup", handleMouseUp);
    };
  }, [dragFrom, movePiece, selectSquare, squareFromPoint]);

  const handlePieceMouseDown = (
    row: number,
    col: number,
    e: React.MouseEvent,
  ) => {
    if (e.button !== 0) return;
    dragStartPixelRef.current = { x: e.clientX, y: e.clientY };
    setDragPosition({ x: e.clientX, y: e.clientY });
    setDragFrom([row, col]);
  };

  return (
    <div
      ref={containerRef}
      className={
        selectedSquare
          ? "absolute inset-0 grid grid-cols-8 grid-rows-8 pointer-events-none"
          : "absolute inset-0 grid grid-cols-8 grid-rows-8"
      }
    >
      {pieces.map(({ kind, color, row, col }) => {
        const isBeingDragged = dragFrom?.[0] === row && dragFrom?.[1] === col;
        const { className, style } = getPieceVisualState(
          isBeingDragged,
          dragPosition,
          row,
          col,
          squareSize,
        );

        return (
          <img
            key={`${row}-${col}`}
            src={PIECE_IMAGES[color][kind]}
            alt={`${color} ${kind}`}
            draggable={false}
            className={className}
            style={style}
            onMouseDown={(e) => handlePieceMouseDown(row, col, e)}
          />
        );
      })}
    </div>
  );
};

export default PieceLayer;
