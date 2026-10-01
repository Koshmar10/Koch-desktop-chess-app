import {
  ChessBishop,
  ChessKing,
  ChessKnight,
  ChessPawn,
  ChessQueen,
  ChessRook,
  type LucideIcon,
} from "lucide-react";
import type { ReactNode } from "react";
import type { PieceColor, PieceKind } from "./lib/types";

// The tile carries the color, so the glyph itself never has to: a light
// tile is a white piece and a dark tile a black one, which stays legible
// at 20px where a white outline on a mid-tone panel would not.
const TILE_BG_CLASS: Record<PieceColor, string> = {
  white: "bg-gray-200 text-gray-900",
  black: "bg-gray-900 text-gray-100",
};

const PIECE_ICON: Record<PieceKind, LucideIcon> = {
  pawn: ChessPawn,
  knight: ChessKnight,
  bishop: ChessBishop,
  rook: ChessRook,
  queen: ChessQueen,
  king: ChessKing,
};

// The piece sits inside its tile rather than filling it, so the tile
// still reads as a piece of board and not as a cropped icon.
const PIECE_ICON_RATIO = 2 / 3;

interface PieceAvatarProps {
  color: PieceColor;
  // Defaults to the king, which is what the player cards and the
  // per-side headings want — they name a side, not a piece.
  kind?: PieceKind;
  // Pixels, not a Tailwind size class: this renders at 48px on the play
  // screen's player cards and at less than half that in the analyzer's
  // panels, and Tailwind can't generate a class from a runtime number.
  size: number;
  // Rendered inside the tile, for anything that overlays it — the play
  // screen hangs a thinking badge off the corner, and the analyzer marks
  // a pawn's weakness the same way.
  badge?: ReactNode;
  title?: string;
}

export const PieceAvatar = ({
  color,
  kind = "king",
  size,
  badge,
  title,
}: PieceAvatarProps) => {
  const Icon = PIECE_ICON[kind];
  const iconSize = size * PIECE_ICON_RATIO;

  return (
    <div
      title={title}
      className={`relative flex shrink-0 items-center justify-center rounded-md shadow-inner ${TILE_BG_CLASS[color]}`}
      style={{ width: size, height: size }}
    >
      <Icon
        style={{ width: iconSize, height: iconSize }}
        className="opacity-80"
      />
      {badge}
    </div>
  );
};
