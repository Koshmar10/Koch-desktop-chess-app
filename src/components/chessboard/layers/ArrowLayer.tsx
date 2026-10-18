import { useCallback, useEffect, useRef, useState } from "react";
import type { Square } from "../../../api/bindings/Square";
import ChessArrow from "./ChessArrow";
import { coordinateFromMouseEvent, squareAtPoint } from "./coordinate";
import { useChessboardContext } from "../ChessboardContext";
import { sameSquare } from "../lib/square";
import { squareName } from "../lib/squareName";
import type { ArrowData } from "../lib/types";

const USER_ARROW_COLOR = "#ff9900";
const RIGHT_MOUSE_BUTTON = 2;

const ArrowLayer = () => {
  const { squareSize, isFlipped } = useChessboardContext();
  const svgRef = useRef<SVGSVGElement>(null);
  const [arrows, setArrows] = useState<ArrowData[]>([]);
  const [startSquare, setStartSquare] = useState<Square | null>(null);

  // A board square, not a screen cell — `ChessArrow` applies the flip
  // itself when it draws.
  const squareUnderPointer = useCallback(
    (e: MouseEvent): Square | null => {
      const bounds = svgRef.current?.getBoundingClientRect();
      if (!bounds) return null;
      return squareAtPoint(
        coordinateFromMouseEvent(e),
        bounds,
        squareSize,
        isFlipped,
      );
    },
    [squareSize, isFlipped],
  );

  // This layer has pointer-events: none (left-clicks must reach the pieces/squares
  // beneath it), so right-click detection listens at the window level instead of on
  // the svg itself — the svg would never receive the event.
  useEffect(() => {
    const handleContextMenu = (e: MouseEvent) => e.preventDefault();

    const handleMouseDown = (e: MouseEvent) => {
      if (e.button !== RIGHT_MOUSE_BUTTON) return;
      setStartSquare(squareUnderPointer(e));
    };

    window.addEventListener("contextmenu", handleContextMenu);
    window.addEventListener("mousedown", handleMouseDown);
    return () => {
      window.removeEventListener("contextmenu", handleContextMenu);
      window.removeEventListener("mousedown", handleMouseDown);
    };
  }, [squareUnderPointer]);

  useEffect(() => {
    if (!startSquare) return;

    const handleMouseUp = (e: MouseEvent) => {
      if (e.button !== RIGHT_MOUSE_BUTTON) return;

      const endSquare = squareUnderPointer(e);
      if (endSquare && !sameSquare(endSquare, startSquare)) {
        setArrows((prev) => [
          ...prev,
          {
            from: startSquare,
            to: endSquare,
            color: USER_ARROW_COLOR,
            type: "user",
          },
        ]);
      } else {
        setArrows([]);
      }
      setStartSquare(null);
    };

    window.addEventListener("mouseup", handleMouseUp);
    return () => window.removeEventListener("mouseup", handleMouseUp);
  }, [startSquare, squareUnderPointer]);

  return (
    <svg
      ref={svgRef}
      className="absolute top-0 left-0 w-full h-full pointer-events-none"
    >
      {arrows.map((arrow, index) => (
        <ChessArrow
          key={`${squareName(arrow.from)}-${squareName(arrow.to)}-${index}`}
          from={arrow.from}
          to={arrow.to}
          color={arrow.color}
          isGhost={arrow.type === "ghost"}
          isFlipped={isFlipped}
          squareSize={squareSize}
        />
      ))}
    </svg>
  );
};

export default ArrowLayer;
