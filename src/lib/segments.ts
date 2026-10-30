import type { Segment } from "../api/bindings/Segment";

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
