import type { LucideIcon } from "lucide-react";
import type { Square } from "../../../api/bindings/Square";
import { PieceAvatar } from "../../../components/chessboard/PieceAvatar";
import {
  DARK_SQUARE_COLOR,
  LIGHT_SQUARE_COLOR,
  isDarkSquare,
} from "../../../components/chessboard/lib/constants";
import { squareName } from "../../../components/chessboard/lib/squareName";
import type { PlacedPiece } from "../../../components/chessboard/lib/types";
import { pieceLabel } from "./pieceLabel";

// The small vocabulary every tab draws findings and moves with — a piece,
// a square, an icon standing in for a word. Shared at the panel level
// because the Position tab draws pins with them and the Engine tab draws
// the best move and the threat with them, and the two should look like
// one language rather than two.

export const PIECE_CHIP_SIZE_PX = 28;

/**
 * A square as a tile, with its name beneath — the piece standing on it
 * if there is one, the bare square in its real board colour if not. The
 * empty form is what lets a move's destination sit beside the piece that
 * moves there as the same kind of object, rather than a tile on one side
 * and a text label on the other.
 *
 * The square name is the one piece of text that stays: it's identity,
 * not prose — with two bishops on the board, the icon alone doesn't say
 * which one a finding is about.
 */
export const TileChip = ({
  square,
  piece,
  showSquare = true,
  title,
}: {
  square: Square;
  piece?: PlacedPiece;
  showSquare?: boolean;
  title?: string;
}) => (
  <span className="relative flex shrink-0">
    {piece ? (
      <PieceAvatar
        color={piece.color}
        kind={piece.kind}
        size={PIECE_CHIP_SIZE_PX}
        title={title}
      />
    ) : (
      <span
        title={title ?? squareName(square)}
        className="shrink-0 rounded-md shadow-inner"
        style={{
          width: PIECE_CHIP_SIZE_PX,
          height: PIECE_CHIP_SIZE_PX,
          backgroundColor: isDarkSquare(square.rank, square.file)
            ? DARK_SQUARE_COLOR
            : LIGHT_SQUARE_COLOR,
        }}
      />
    )}
    {/* Hung below the tile rather than stacked under it, so it's no part
        of the chip's box: every chip is exactly one tile tall, captioned
        or not. In flow, a caption made its chip taller, and `items-center`
        then lifted captioned tiles above uncaptioned ones beside them —
        a pin's rider sat lower than the pieces at either end, and arrows
        centred on tile-plus-caption instead of on the tiles. Rows holding
        captioned chips reserve the space underneath with bottom padding. */}
    {showSquare && (
      <span className="pointer-events-none absolute top-full left-1/2 mt-0.5 -translate-x-1/2 font-mono text-xs leading-none text-foreground/45">
        {squareName(square)}
      </span>
    )}
  </span>
);

/**
 * A piece named by id, which is how every analyzer finding refers to
 * one. Renders nothing for an id the position doesn't hold — that means
 * the findings and the board are a ply apart, and a placeholder would
 * hide it.
 */
export const PieceChip = ({
  id,
  pieces,
  showSquare = true,
}: {
  id: number;
  pieces: PlacedPiece[];
  showSquare?: boolean;
}) => {
  const piece = pieces.find((p) => p.id === id);
  if (!piece) return null;

  return (
    <TileChip
      square={piece.square}
      piece={piece}
      showSquare={showSquare}
      title={pieceLabel(id, pieces) ?? undefined}
    />
  );
};

export const SquareChip = ({ label }: { label: string }) => (
  <span className="flex h-5 min-w-5 items-center justify-center rounded-sm border-[1px] border-border/60 bg-card/40 px-1 font-mono text-xs text-foreground/65">
    {label}
  </span>
);

/** An icon standing in for a label, with the word kept as its tooltip. */
export const IconLabel = ({
  icon: Icon,
  title,
}: {
  icon: LucideIcon;
  title: string;
}) => (
  <span title={title} className="flex shrink-0 text-foreground/40">
    <Icon size={14} />
  </span>
);
