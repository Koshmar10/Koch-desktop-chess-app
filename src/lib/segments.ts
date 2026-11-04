import type { PieceView } from "../api/bindings/PieceView";
import type { Segment } from "../api/bindings/Segment";
import type { SegmentMove } from "../api/bindings/SegmentMove";

/**
 * A move from an answer, shown on the board where it's played: the board
 * before it, the move, and the moves of its line that led there, which
 * stay drawn faded so a line stepped through shows how it got here.
 */
export interface MovePreview {
  pieces: PieceView[];
  uci: string;
  // UCI, earliest first.
  trail: string[];
}

// The preview for `moves[index]` of a line (a single move is a line of
// one). Null for a move that isn't verified: there's no board to show it on.
export const movePreview = (
  moves: readonly SegmentMove[],
  index: number,
): MovePreview | null => {
  const move = moves[index];
  if (!move || move.uci === null || move.pieces_before.length === 0) {
    return null;
  }
  return {
    pieces: move.pieces_before,
    uci: move.uci,
    trail: moves
      .slice(0, index)
      .flatMap((earlier) => (earlier.uci === null ? [] : [earlier.uci])),
  };
};

// Scheme of the link that stands in for a move tag while the reply goes
// through the markdown renderer.
export const SEGMENT_LINK_PREFIX = "koch-segment:";

// A reply's segments as one markdown string: text as it is, every move tag
// a link to its position in `segments`, so `Markdown` can swap in a chip
// and the surrounding text keeps flowing around it.
export const segmentsToMarkdown = (segments: Segment[]): string =>
  segments
    .map((segment, index) =>
      segment.kind === "Text"
        ? segment.text
        : `[chip](${SEGMENT_LINK_PREFIX}${index})`,
    )
    .join("");

// Which segment a link stands for, or null for an ordinary link.
export const segmentIndexOf = (href: string | undefined): number | null => {
  if (!href?.startsWith(SEGMENT_LINK_PREFIX)) return null;
  const index = Number(href.slice(SEGMENT_LINK_PREFIX.length));
  return Number.isInteger(index) ? index : null;
};
