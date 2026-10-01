import { useChessboardContext } from "../ChessboardContext";
import { TONE_SQUARE_COLOR } from "../lib/highlightTones";
import { flipCoords } from "../lib/orientation";
import type { ArrowData, SquareHighlight, SquareTint } from "../lib/types";
import ChessArrow from "./ChessArrow";

interface HighlightLayerProps {
  highlights: SquareHighlight[];
  // Kept separate from `highlights` rather than read off them, so arrows
  // that belong to no finding — the engine's best move, the opponent's
  // threat (KOCH-14) — can ride this layer too.
  arrows?: ArrowData[];
  // A full-board shading (square control) — drawn beneath the highlights,
  // so a finding you've lit still stands out on a tinted board.
  tints?: SquareTint[];
}

/**
 * Squares and arrows drawn for some external reason — a pin's ray, a
 * fork's victims, an engine suggestion.
 *
 * Driven entirely by its props and stateless by design: it knows about
 * squares, arrows and tones, never about pins or forks, so a new kind of
 * finding needs no change here. Distinct from the legal-move dots in
 * `Squares`, which are interaction-driven rather than state-driven, and
 * from `ArrowLayer`, which owns the user's own right-click arrows.
 *
 * Mount it between `Squares` and `PieceLayer` so everything it draws sits
 * over the board but under the pieces — a piece hidden by the highlight
 * that points at it is the one thing you were looking for.
 */
const HighlightLayer = ({
  highlights,
  arrows = [],
  tints = [],
}: HighlightLayerProps) => {
  const { squareSize, isFlipped } = useChessboardContext();

  // Board (rank/file) square to its on-screen box.
  const boxOf = (rank: number, file: number) => {
    const [row, col] = flipCoords(rank, file, isFlipped);
    return {
      left: col * squareSize,
      top: row * squareSize,
      width: squareSize,
      height: squareSize,
    };
  };

  return (
    <>
      <div className="pointer-events-none absolute inset-0">
        {tints.map(({ square, color }) => (
          <div
            key={`tint-${square.rank}-${square.file}`}
            className="absolute transition-colors duration-150"
            style={{
              ...boxOf(square.rank, square.file),
              backgroundColor: color,
            }}
          />
        ))}
        {highlights.flatMap(({ id, squares, tone }) =>
          squares.map((square) => (
            <div
              key={`${id}-${square.rank}-${square.file}`}
              className="absolute transition-opacity duration-150"
              style={{
                ...boxOf(square.rank, square.file),
                backgroundColor: TONE_SQUARE_COLOR[tone],
              }}
            />
          )),
        )}
      </div>

      {/* z-10 lifts the arrows above `PieceLayer`, which sits after this
          layer in the DOM but paints at the default z-index. The fills
          above stay below the pieces and the arrows here stay above them:
          a piece should never be hidden by its own highlight, but a line
          joining two pieces shouldn't be broken by everything standing
          between them either. A piece being dragged is z-50, so it still
          comes out on top. */}
      {arrows.length > 0 && (
        <svg className="pointer-events-none absolute top-0 left-0 z-10 h-full w-full">
          {arrows.map((arrow) => (
            <ChessArrow
              key={`${arrow.from[0]}-${arrow.from[1]}-${arrow.to[0]}-${arrow.to[1]}`}
              from={arrow.from}
              to={arrow.to}
              color={arrow.color}
              isGhost={arrow.type === "ghost"}
              isFlipped={isFlipped}
              squareSize={squareSize}
            />
          ))}
        </svg>
      )}
    </>
  );
};

export default HighlightLayer;
