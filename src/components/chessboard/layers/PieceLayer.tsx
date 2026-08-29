import { useCallback, useEffect, useRef, useState } from "react";
import { useChessboardContext } from "../ChessboardContext";
import { flipCoords } from "../lib/orientation";
import { PIECE_IMAGES } from "../lib/pieceImages";
import type { PieceColor } from "../lib/types";
import { getPieceVisualState } from "./pieceVisualState";

// Below this drag distance, a mousedown+mouseup on a piece is treated as a click (select) rather than a drag (move).
const CLICK_THRESHOLD_PX = 6;

const PieceLayer = () => {
  const {
    pieces,
    squareSize,
    boardSize,
    isFlipped,
    selectedSquare,
    selectSquare,
    movePiece,
    clearSelection,
    canPlayerMove,
    humanColor,
  } = useChessboardContext();

  const containerRef = useRef<HTMLDivElement>(null);
  const dragStartPixelRef = useRef<{ x: number; y: number } | null>(null);
  // Whether the piece being picked up was already the selection *before*
  // this mousedown — distinguishes "click it again to deselect" from
  // "click a different piece to select it", since mousedown now selects
  // unconditionally (needed so legal-move highlights show up immediately
  // once a drag starts, not only after a completed click).
  const wasAlreadySelectedRef = useRef(false);
  // Board (rank/file) coordinates, not screen coordinates — same space as
  // `selectedSquare`/`movePiece`.
  const [dragFrom, setDragFrom] = useState<[number, number] | null>(null);
  const [dragPosition, setDragPosition] = useState<{
    x: number;
    y: number;
  } | null>(null);

  const canDragPiece = useCallback(
    (color: PieceColor) => canPlayerMove && color === humanColor,
    [canPlayerMove, humanColor],
  );

  // Converts a pixel position to a board square, undoing the flip so the
  // result is directly comparable to `dragFrom`/usable with `movePiece`.
  const squareFromPoint = useCallback(
    (clientX: number, clientY: number): [number, number] | null => {
      const bounds = containerRef.current?.getBoundingClientRect();
      if (!bounds) return null;
      const screenCol = Math.floor((clientX - bounds.left) / squareSize);
      const screenRow = Math.floor((clientY - bounds.top) / squareSize);
      if (
        screenRow < 0 ||
        screenRow >= boardSize ||
        screenCol < 0 ||
        screenCol >= boardSize
      )
        return null;
      return flipCoords(screenRow, screenCol, isFlipped);
    },
    [boardSize, squareSize, isFlipped],
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
        // Selection already happened on mousedown — a plain click just
        // toggles it off if it was already the selection.
        if (wasAlreadySelectedRef.current) {
          clearSelection();
        }
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
  }, [dragFrom, movePiece, clearSelection, squareFromPoint]);

  const handlePieceMouseDown = (
    rank: number,
    file: number,
    color: PieceColor,
    e: React.MouseEvent,
  ) => {
    if (e.button !== 0 || !canDragPiece(color)) return;
    wasAlreadySelectedRef.current =
      selectedSquare?.[0] === rank && selectedSquare?.[1] === file;
    dragStartPixelRef.current = { x: e.clientX, y: e.clientY };
    setDragPosition({ x: e.clientX, y: e.clientY });
    setDragFrom([rank, file]);
    // Selecting immediately (not just on a completed click) means legal
    // moves are highlighted from the moment the drag starts, and picking
    // up a different own piece switches the selection right away instead
    // of dragging it around under the old selection.
    selectSquare(rank, file);
  };

  return (
    <div
      ref={containerRef}
      className="absolute inset-0 grid grid-cols-8 grid-rows-8"
    >
      {pieces.map(({ id, kind, color, square }) => {
        const { rank, file } = square;
        const [screenRow, screenCol] = flipCoords(rank, file, isFlipped);
        const isBeingDragged =
          dragFrom?.[0] === rank && dragFrom?.[1] === file;
        const { className, style } = getPieceVisualState(
          isBeingDragged,
          dragPosition,
          screenRow,
          screenCol,
          squareSize,
        );
        // Only the player's own, currently-movable pieces stay interactive
        // here — everything else (opponent pieces, or all pieces when it's
        // not a legal moment to move) is pointer-events-none so clicks on
        // them fall through to Squares (e.g. clicking a capture square).
        const pieceClassName = canDragPiece(color)
          ? className
          : `${className} pointer-events-none`;

        return (
          <img
            key={id}
            src={PIECE_IMAGES[color][kind]}
            alt={`${color} ${kind}`}
            draggable={false}
            className={pieceClassName}
            style={style}
            onMouseDown={(e) => handlePieceMouseDown(rank, file, color, e)}
          />
        );
      })}
    </div>
  );
};

export default PieceLayer;
