import type { MoveQuality } from "../../api/bindings/MoveQuality";

/**
 * One colour per grade, ordered so the set reads as a scale rather than
 * seven unrelated colours: cool for the best moves, through greens for
 * the sound ones, to yellow, orange and red as the mistakes get worse —
 * so a run of dots down the move list shows where a game went wrong
 * without reading a single label.
 *
 * Shared at the analyzer's root rather than kept in the move list, since
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
