// What the Analyzer screen renders. These live here rather than in
// `api/bindings/` because KOCH-10 — the backend session that will emit
// them — doesn't exist yet: the screen is being built against mock data
// first, so these are the contract the Rust side has to generate *into*,
// not the other way round. When KOCH-10 lands, ts-rs replaces this file
// with bindings carrying the same field names.

/**
 * One engine score, always from White's perspective: a positive
 * `centipawns` favours White no matter whose turn it is.
 *
 * There is deliberately no "is from white perspective" flag. The old app
 * carried one, set from the *pre*-normalisation condition, so it read
 * `false` whenever Black was to move despite the value already being
 * White-relative — honouring it double-flipped the sign. See
 * KOCH-HANDOFF.md §3; the fix is that the flag doesn't exist to honour.
 */
export type EvalScore =
  | { kind: "cp"; centipawns: number }
  /**
   * Plies-to-mate, signed by who delivers it: positive means White mates,
   * negative means Black does. Same White-relative frame as `centipawns`,
   * not UCI's side-to-move frame.
   */
  | { kind: "mate"; movesToMate: number };

export interface PvLine {
  /** Stockfish's own 1-based multipv rank, carried through as-is. */
  multipv: number;
  score: EvalScore;
  /** UCI tokens, e.g. `["e2e4", "e7e5"]` — SAN conversion is a later ticket. */
  moves: string[];
}

export interface EngineSnapshot {
  depth: number;
  /**
   * Best line first, exactly as the engine ranked them — an **array**, not
   * the keyed `Record<number, …>` the old bindings used. That record is
   * what tempted `updateSuggestion` into re-ranking by raw score, which
   * picks the *worst* line whenever Black is to move (KOCH-HANDOFF.md §3).
   * Nothing downstream may re-sort this.
   */
  lines: PvLine[];
}

/**
 * One value rather than the old `engineRunning` + `engineLoading` pair,
 * which could contradict each other (running *and* loading) with no
 * defined meaning for the combination.
 */
export type EngineStatus = "running" | "stopped" | "loading";

/** The board overlays the user can toggle on. */
/**
 * Board overlays that are switched on or off as a whole, rather than per
 * finding. Each is *derived* from current data on every render rather
 * than stored: the best move changes as the search deepens, so storing
 * the arrow would freeze it at whatever it was when you clicked.
 *
 * Per-finding highlights (one pin, one fork) are `SquareHighlight`s and
 * live separately — these are the three with nothing to hang a row on.
 */
export type Overlay = "bestMove" | "threat" | "control";

export type ActiveOverlays = Record<Overlay, boolean>;

export const NO_OVERLAYS_ACTIVE: ActiveOverlays = {
  bestMove: false,
  threat: false,
  control: false,
};

/** Viewed-ply value for "before any move was played". */
export const START_POSITION_PLY = -1;
