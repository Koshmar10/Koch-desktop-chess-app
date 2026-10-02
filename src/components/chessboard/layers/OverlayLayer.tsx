import { useChessboardContext } from "../ChessboardContext";
import { flipCoords } from "../lib/orientation";
import type { OverlayMask } from "../lib/types";
import { OVERLAY_ARROWS_Z_INDEX } from "../lib/zIndex";
import ChessArrow from "./ChessArrow";

interface OverlayLayerProps {
  masks: OverlayMask[];
}

/**
 * Everything drawn on the board for some external reason — a pin, a
 * fork, square control, an engine move — as a list of masks.
 *
 * Driven entirely by its props and stateless by design: it knows about
 * squares, arrows and paint order, never about pins or forks, so a new
 * kind of finding needs no change here. Distinct from the legal-move dots
 * in `Squares`, which are interaction-driven rather than state-driven,
 * and from `ArrowLayer`, which owns the user's own right-click arrows.
 *
 * Draws by plane, not by mask: every mask's fills go under the pieces and
 * every mask's arrows over them, whatever their priority. Priority only
 * orders masks *within* a plane — a mask drawing itself whole would put
 * its arrows under a higher mask's fills, or its fills over the pieces.
 *
 * Mount it between `Squares` and `PieceLayer`.
 */
const OverlayLayer = ({ masks }: OverlayLayerProps) => {
  const { squareSize, isFlipped } = useChessboardContext();

  // Lowest first: later siblings paint over earlier ones, so the higher
  // priority ends up on top. Copied before sorting — the list is the
  // caller's.
  const ordered = [...masks].sort((a, b) => a.priority - b.priority);
  const arrows = ordered.flatMap(({ id, arrows = [] }) =>
    arrows.map((arrow, index) => ({ key: `${id}-${index}`, arrow })),
  );

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
      {/* Under the pieces: a piece should never be hidden by the very
          fill that points at it. */}
      <div className="pointer-events-none absolute inset-0">
        {ordered.flatMap(({ id, squares = [] }) =>
          squares.map(({ square, color }) => (
            <div
              key={`${id}-${square.rank}-${square.file}`}
              className="absolute transition-colors duration-150"
              style={{
                ...boxOf(square.rank, square.file),
                backgroundColor: color,
              }}
            />
          )),
        )}
      </div>

      {/* Raised above `PieceLayer`, which sits after this layer in the
          DOM but has no z-index of its own: a line joining two pieces
          shouldn't be broken by everything standing between them. The
          dragged piece sits higher still — see `lib/zIndex.ts` for the
          whole order. */}
      {arrows.length > 0 && (
        <svg
          className="pointer-events-none absolute top-0 left-0 h-full w-full"
          style={{ zIndex: OVERLAY_ARROWS_Z_INDEX }}
        >
          {arrows.map(({ key, arrow }) => (
            <ChessArrow
              key={key}
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

      {/* `ghostPieces` aren't drawn yet — nothing builds them. */}
    </>
  );
};

export default OverlayLayer;
