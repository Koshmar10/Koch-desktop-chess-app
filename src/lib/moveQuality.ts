import type { MoveQuality } from "../api/bindings/MoveQuality";

/**
 * One colour per grade, ordered so the set reads as a scale rather than
 * seven unrelated colours: cool for the best moves, through greens for
 * the sound ones, to yellow, orange and red as the mistakes get worse —
 * so a run of dots down the move list shows where a game went wrong
 * without reading a single label.
 *
 * Kept outside any one screen: the shared move list draws these dots, and
 * the timeline strip's per-ply markers are meant to use the same colours.
 * A `Record` over the binding's union, so a grade added in the backend
 * fails to compile here until it has a colour.
 */
export const MOVE_QUALITY_COLOR: Record<MoveQuality, string> = {
  Brilliant: "#1baca6",
  Great: "#4f8fca",
  Excellent: "#5f9e3b",
  Good: "#9ab66b",
  Inaccuracy: "#e3b324",
  Mistake: "#e2802b",
  Blunder: "#d1352f",
};

/** Best to worst — the order a breakdown lists them in. */
export const MOVE_QUALITY_ORDER: readonly MoveQuality[] = [
  "Brilliant",
  "Great",
  "Excellent",
  "Good",
  "Inaccuracy",
  "Mistake",
  "Blunder",
];

/** What a count of each is called in a breakdown — "3 Mistakes". */
export const MOVE_QUALITY_COUNT_LABEL: Record<MoveQuality, string> = {
  Brilliant: "Brilliant",
  Great: "Great",
  Excellent: "Excellent",
  Good: "Good",
  Inaccuracy: "Inaccuracies",
  Mistake: "Mistakes",
  Blunder: "Blunders",
};

/**
 * How many of `qualities` got each grade — every grade present, at zero
 * when nothing got it, so a breakdown always has the same rows.
 */
export const countByQuality = (
  qualities: readonly MoveQuality[],
): Record<MoveQuality, number> => {
  // Spelled out rather than built from a list: as a `Record` over the
  // binding's union, a grade added in the backend fails to compile here.
  const counts: Record<MoveQuality, number> = {
    Brilliant: 0,
    Great: 0,
    Excellent: 0,
    Good: 0,
    Inaccuracy: 0,
    Mistake: 0,
    Blunder: 0,
  };
  for (const quality of qualities) counts[quality] += 1;
  return counts;
};
