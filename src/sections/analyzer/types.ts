// What the Analyzer screen renders. The engine's output types are generated
// by ts-rs from the live engine's Rust structs (KOCH-10) and re-exported
// here, so they can't drift from what the backend actually sends.
//
// Scores are always from White's point of view — the backend flips them
// once, at the source (KOCH-HANDOFF.md §3) — and `pv_lines` arrive best
// first in Stockfish's own order. Never re-sort them by score: with Black
// to move, the "highest" line is the worst one.
export type { EvalScore } from "../../api/bindings/EvalScore";
export type { PvLine } from "../../api/bindings/PvLine";
export type { EngineSnapshot } from "../../api/bindings/EngineSnapshot";

/**
 * One value rather than the old `engineRunning` + `engineLoading` pair,
 * which could contradict each other (running *and* loading) with no
 * defined meaning for the combination.
 */
export type EngineStatus = "running" | "stopped" | "loading";

/** Viewed-ply value for "before any move was played". */
export const START_POSITION_PLY = -1;

/**
 * Which overlay masks are on the board, by id, and the way to change
 * that — bundled so the panel's rows can read it as one value, from
 * `MaskSelectionContext` (see `panel/maskSelection.ts`).
 *
 * Ids only, never the masks themselves: a mask's squares belong to one
 * ply, its id to the reason it's drawn. The masks are rebuilt from the
 * viewed ply on every render, so whatever's switched on follows you
 * through the game (see `overlays/masks.ts`).
 */
export interface MaskSelection {
  shown: ReadonlySet<string>;
  /**
   * The row the pointer is resting on. Separate from `shown` and
   * transient by design: it never changes what's switched on, so moving
   * the pointer away always puts the board back as the clicks left it.
   */
  previewId: string | null;
  onToggle: (id: string) => void;
  onPreview: (id: string | null) => void;
}
