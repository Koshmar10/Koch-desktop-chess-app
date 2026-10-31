import type { AnalysisStage } from "../../api/bindings/AnalysisStage";

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
 * Which screen the analyzer is: a saved game, or the sandbox. It's the
 * route, not stored state (see `AnalyzerProvider`).
 */
export type AnalyzerMode = "game" | "sandbox";

/**
 * Ordered widest scope first: the game as a whole, then the position in
 * front of you, then what the engine makes of it.
 *
 * Engine and Position are also split by *what changes them* — Engine
 * repaints several times a second while a search runs, Position only when
 * the viewed ply changes. Interleaving those in one scroll means half the
 * panel is always flickering under the half you're reading.
 */
export type PanelTab = "game" | "position" | "engine";

/**
 * One point on the eval timeline: a move with a stored eval. `index` is the
 * move's ply counted from 1, as the axis shows it — `goToPly` takes
 * `index - 1` — and `score` is White-relative centipawns.
 */
export interface TimelineMove {
  index: number;
  score: number;
}

// The toolbar's mode-specific buttons. A null handler means the action has
// nothing to act on right now, and its button shows disabled.
export interface GameActions {
  analysisStage: AnalysisStage | undefined;
  // Null for an import that's never been analysed: it has no side to grade
  // yet, and History is where that gets picked.
  onReanalyze: (() => void) | null;
  onTryInSandbox: (() => void) | null;
  onNextMistake: (() => void) | null;
}

export interface SandboxActions {
  onReset: () => void;
  onUndo: (() => void) | null;
}

/**
 * One value rather than the old `engineRunning` + `engineLoading` pair,
 * which could contradict each other (running *and* loading) with no
 * defined meaning for the combination.
 */
export type EngineStatus = "running" | "stopped" | "loading";

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
