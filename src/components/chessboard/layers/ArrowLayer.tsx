import { useCallback, useEffect, useRef, useState } from "react";
import ChessArrow from "./ChessArrow";
import { useChessboardContext } from "../ChessboardContext";
import { flipCoords } from "../lib/orientation";
import type { ArrowData } from "../lib/types";

const USER_ARROW_COLOR = "#ff9900";
const RIGHT_MOUSE_BUTTON = 2;

const ArrowLayer: React.FC = () => {
  const { squareSize, boardSize, isFlipped } = useChessboardContext();
  const svgRef = useRef<SVGSVGElement>(null);
  const [arrows, setArrows] = useState<ArrowData[]>([]);
  const [startSquare, setStartSquare] = useState<[number, number] | null>(null);

  // Returns a board (rank/file) square — `ChessArrow` does its own
  // domain -> screen conversion via `isFlipped`, so this undoes the pixel
  // math's screen coordinates first.
  const squareFromPoint = useCallback(
    (clientX: number, clientY: number): [number, number] | null => {
      const bounds = svgRef.current?.getBoundingClientRect();
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

  // This layer has pointer-events: none (left-clicks must reach the pieces/squares
  // beneath it), so right-click detection listens at the window level instead of on
  // the svg itself — the svg would never receive the event.
  useEffect(() => {
    const handleContextMenu = (e: MouseEvent) => e.preventDefault();

    const handleMouseDown = (e: MouseEvent) => {
      if (e.button !== RIGHT_MOUSE_BUTTON) return;
      setStartSquare(squareFromPoint(e.clientX, e.clientY));
    };

    window.addEventListener("contextmenu", handleContextMenu);
    window.addEventListener("mousedown", handleMouseDown);
    return () => {
      window.removeEventListener("contextmenu", handleContextMenu);
      window.removeEventListener("mousedown", handleMouseDown);
    };
  }, [boardSize, squareSize, squareFromPoint]);

  useEffect(() => {
    if (!startSquare) return;

    const handleMouseUp = (e: MouseEvent) => {
      if (e.button !== RIGHT_MOUSE_BUTTON) return;

      const endSquare = squareFromPoint(e.clientX, e.clientY);
      if (
        endSquare &&
        (endSquare[0] !== startSquare[0] || endSquare[1] !== startSquare[1])
      ) {
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
  }, [startSquare, boardSize, squareSize, squareFromPoint]);

  return (
    <svg
      ref={svgRef}
      className="absolute top-0 left-0 w-full h-full pointer-events-none"
    >
      {arrows.map((arrow, index) => (
        <ChessArrow
          key={`${arrow.from[0]}-${arrow.from[1]}-${arrow.to[0]}-${arrow.to[1]}-${index}`}
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
