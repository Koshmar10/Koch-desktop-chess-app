import { getSquareCenter } from "./squareCenter";

interface ChessArrowProps {
  from: [number, number]; // [row, col]
  to: [number, number]; // [row, col]
  color?: string;
  isGhost?: boolean;
  opacity?: number;
  isFlipped?: boolean;
  squareSize: number;
}

const ChessArrow: React.FC<ChessArrowProps> = ({
  from,
  to,
  color = "#ff9900",
  opacity = 1.0,
  isFlipped = false,
  isGhost = false,
  squareSize,
}) => {
  const start = getSquareCenter(from, isFlipped, squareSize);
  const end = getSquareCenter(to, isFlipped, squareSize);

  const markerId = `arrowhead-${color.replace("#", "")}`;
  const strokeWidth = squareSize * 0.2;

  const effectiveOpacity = isGhost ? Math.min(opacity, 0.5) : opacity;

  return (
    <>
      <defs>
        <marker
          id={markerId}
          markerWidth={2.5}
          markerHeight={2.5}
          refX={1.25}
          refY={1.25}
          orient="auto"
        >
          <path d="M0,0 L2.5,1.25 L0,2.5" fill={color} />
        </marker>
      </defs>

      <line
        x1={start.x}
        y1={start.y}
        x2={end.x}
        y2={end.y}
        stroke={color}
        strokeWidth={strokeWidth}
        markerEnd={`url(#${markerId})`}
        opacity={effectiveOpacity}
        strokeLinecap="round"
      />
    </>
  );
};

export default ChessArrow;
