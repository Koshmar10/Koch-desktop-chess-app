import { useCallback, useEffect, useRef, useState } from "react";
import { useChessboardContext } from "../ChessboardContext";
import { flipCoords, squaresEqual } from "../lib/orientation";
import { PIECE_IMAGES } from "../lib/pieceImages";
import type { PieceColor } from "../lib/types";
import { getPieceVisualState } from "./pieceVisualState";
import { coordinateFromMouseEvent, type Coordinate } from "./coordinate";

// Below this drag distance, a mousedown+mouseup on a piece is treated as a click (select) rather than a drag (move).
const CLICK_THRESHOLD_PX = 6;

// The piece in your hand. `from` is in board (rank/file) coordinates, not
// screen coordinates — same space as `selectedSquare`/`movePiece`.
interface Drag {
  pieceId: number;
  from: [number, number];
}

// The piece most recently let go of after a drag.
interface Release {
  pieceId: number;
  // Counts releases, and goes into the released piece's key — so every
  // release puts the piece down as a brand-new <img>. A new element has no
  // previous position for its slide to start from, so it lands where it was
  // dropped instead of sliding in from wherever the drag left the old one.
  count: number;
  // Where a legally dropped piece is shown until the board's next position
  // arrives. `movePiece` usually sends the move off to the backend, so for a
  // moment the board still has the piece on its old square — without this
  // it would snap back there, then slide over once the answer came in. Null
  // when the drop wasn't a legal move, and once that next position is in.
  landingSquare: [number, number] | null;
}

const PieceLayer = () => {
  const {
    pieces,
    squareSize,
    boardSize,
    isFlipped,
    selectedSquare,
    legalDestinations,
    selectSquare,
    movePiece,
    clearSelection,
    canPlayerMove,
    humanColor,
  } = useChessboardContext();

  const containerRef = useRef<HTMLDivElement>(null);
  const dragStartPixelRef = useRef<Coordinate | null>(null);
  // Whether the piece being picked up was already the selection *before*
  // this mousedown — distinguishes "click it again to deselect" from
  // "click a different piece to select it", since mousedown now selects
  // unconditionally (needed so legal-move highlights show up immediately
  // once a drag starts, not only after a completed click).
  const wasAlreadySelectedRef = useRef(false);
  const [drag, setDrag] = useState<Drag | null>(null);
  const [dragPosition, setDragPosition] = useState<Coordinate | null>(null);
  const [release, setRelease] = useState<Release | null>(null);

  // Adjusting state during render, as Chessboard does for its pieces: a new
  // position is the answer a landing piece was waiting for, so from here on
  // it's drawn on its real square.
  const [prevPieces, setPrevPieces] = useState(pieces);
  if (pieces !== prevPieces) {
    setPrevPieces(pieces);
    if (release?.landingSquare) {
      setRelease({ ...release, landingSquare: null });
    }
  }

  const canDragPiece = useCallback(
    (color: PieceColor) => canPlayerMove && color === humanColor,
    [canPlayerMove, humanColor],
  );

  // Converts a pixel position to a board square, undoing the flip so the
  // result is directly comparable to `drag.from`/usable with `movePiece`.
  const squareFromPoint = useCallback(
    (clientX: number, clientY: number): [number, number] | null => {
      const bounds = containerRef.current?.getBoundingClientRect();
      if (!bounds) return null;
      const screenCol = Math.floor((clientX - bounds.left) / squareSize);
      const screenRow = Math.floor((clientY - bounds.top) / squareSize);

      const rowInBounds = screenRow >= 0 && screenRow < boardSize;
      const colInBounds = screenCol >= 0 && screenCol < boardSize;

      if (!rowInBounds || !colInBounds) return null;

      return flipCoords(screenRow, screenCol, isFlipped);
    },
    [boardSize, squareSize, isFlipped],
  );

  const handlePieceMouseDown = (
    pieceId: number,
    rank: number,
    file: number,
    color: PieceColor,
    e: React.MouseEvent,
  ) => {
    if (e.button !== 0 || !canDragPiece(color)) return;

    wasAlreadySelectedRef.current = squaresEqual(selectedSquare, [rank, file]);
    const start = coordinateFromMouseEvent(e);

    dragStartPixelRef.current = start;
    setDragPosition(start);
    setDrag({ pieceId, from: [rank, file] });
    // Selecting immediately (not just on a completed click) means legal
    // moves are highlighted from the moment the drag starts, and picking
    // up a different own piece switches the selection right away instead
    // of dragging it around under the old selection.
    selectSquare(rank, file);
  };

  useEffect(() => {
    if (!drag) return;
    const { pieceId, from } = drag;

    const handleMouseMove = (e: MouseEvent) => {
      setDragPosition(coordinateFromMouseEvent(e));
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
      } else if (to && !squaresEqual(to, from)) {
        movePiece(from, to);
      }

      // `legalDestinations` is the dragged piece's, since mousedown
      // selected it.
      const isLegalDrop =
        movedDistance >= CLICK_THRESHOLD_PX &&
        to !== null &&
        legalDestinations.some((sq) => sq.rank === to[0] && sq.file === to[1]);

      dragStartPixelRef.current = null;
      setDragPosition(null);
      setDrag(null);
      setRelease((previous) => ({
        pieceId,
        count: (previous?.count ?? 0) + 1,
        landingSquare: isLegalDrop ? to : null,
      }));
    };

    window.addEventListener("mousemove", handleMouseMove);
    window.addEventListener("mouseup", handleMouseUp);
    return () => {
      window.removeEventListener("mousemove", handleMouseMove);
      window.removeEventListener("mouseup", handleMouseUp);
    };
  }, [drag, movePiece, clearSelection, squareFromPoint, legalDestinations]);

  const landingSquare = release?.landingSquare ?? null;

  return (
    <div
      ref={containerRef}
      // Keyed on orientation, so flipping the board remounts every piece.
      // Fresh elements have nothing to slide from, so the pieces jump to
      // their mirrored squares instead of all crossing the board at once.
      key={isFlipped ? "flipped" : "upright"}
      // pointer-events-none so clicks on empty squares / capture targets fall
      // through to the Squares layer beneath (which owns click-to-move). Only
      // the draggable pieces below opt back in, via pointer-events-auto.
      className="absolute inset-0 pointer-events-none"
    >
      {/* Drawn in id order, not the order `pieces` comes in — the backend
          lists pieces square by square, so every move reshuffles it. React
          would follow a reshuffle by moving <img>s around the DOM, and an
          element taken out and put back loses the position its slide
          starts from, so the move would jump instead. */}
      {[...pieces]
        .sort((a, b) => a.id - b.id)
        .map(({ id, kind, color, square }) => {
          const isReleased = release !== null && id === release.pieceId;
          // Whatever stands on the square a piece is landing on is being
          // captured — leave it out now, rather than draw the two on top of
          // each other until the next position confirms it.
          if (
            !isReleased &&
            squaresEqual(landingSquare, [square.rank, square.file])
          ) {
            return null;
          }

          const [rank, file]: [number, number] =
            isReleased && landingSquare
              ? landingSquare
              : [square.rank, square.file];
          const [screenRow, screenCol] = flipCoords(rank, file, isFlipped);
          const { className, style } = getPieceVisualState(
            drag?.pieceId === id,
            dragPosition,
            screenRow,
            screenCol,
            squareSize,
          );
          // The container is pointer-events-none; only the player's own,
          // currently-movable pieces opt back in so they can be picked up.
          // Everything else stays non-interactive, so clicks on it fall
          // through to Squares (e.g. clicking a capture square).
          const pieceClassName = canDragPiece(color)
            ? `${className} pointer-events-auto`
            : className;

          return (
            <img
              key={isReleased ? `${id}:${release.count}` : id}
              src={PIECE_IMAGES[color][kind]}
              alt={`${color} ${kind}`}
              draggable={false}
              className={pieceClassName}
              style={style}
              onMouseDown={(e) =>
                handlePieceMouseDown(id, rank, file, color, e)
              }
            />
          );
        })}
    </div>
  );
};

export default PieceLayer;
