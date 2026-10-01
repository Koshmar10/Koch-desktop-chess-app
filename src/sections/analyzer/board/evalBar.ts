import type { EvalScore } from "../types";

const CENTIPAWNS_PER_PAWN = 100;

/**
 * Past this many pawns the bar is pinned to one end anyway, so mapping the
 * full centipawn range onto it would waste almost all the travel on
 * positions nobody needs a bar to tell them about.
 */
const BAR_CLAMP_PAWNS = 5;

/** Shown instead of a number before the engine has reported anything. */
export const EVAL_UNKNOWN_LABEL = "—";

const clamp = (value: number, min: number, max: number): number =>
  Math.min(max, Math.max(min, value));

const signPrefix = (value: number): string => {
  if (value > 0) return "+";
  if (value < 0) return "-";
  return "";
};

/**
 * White's share of the eval bar, 0 (Black is winning) to 1 (White is).
 * An unknown score sits at dead even rather than at either end — the bar
 * has to render *something* before the first `info` line arrives, and a
 * level bar is the one reading that claims nothing.
 */
export const evalBarWhiteFraction = (score: EvalScore | null): number => {
  if (score === null) return 0.5;
  if (score.kind === "mate") return score.movesToMate > 0 ? 1 : 0;

  const pawns = clamp(
    score.centipawns / CENTIPAWNS_PER_PAWN,
    -BAR_CLAMP_PAWNS,
    BAR_CLAMP_PAWNS,
  );
  return 0.5 + pawns / (2 * BAR_CLAMP_PAWNS);
};

/** The score as it reads on the bar: `+1.25`, `-0.30`, `+M3`, `0.00`. */
export const formatEvalScore = (score: EvalScore | null): string => {
  if (score === null) return EVAL_UNKNOWN_LABEL;
  if (score.kind === "mate") {
    return `${signPrefix(score.movesToMate)}M${Math.abs(score.movesToMate)}`;
  }

  const pawns = score.centipawns / CENTIPAWNS_PER_PAWN;
  return `${signPrefix(pawns)}${Math.abs(pawns).toFixed(2)}`;
};

/**
 * The score the bar and the toolbar read, which is always the *top* line's
 * — multipv lines arrive best-first, so this is index 0 and never a
 * scan for the highest number. The old app's eval bar got this right via
 * `getFirstPvLine` while its suggestion arrow didn't; keeping the rule in
 * one exported function means both callers get the same answer.
 */
export const topLineScore = (
  lines: { score: EvalScore }[] | undefined,
): EvalScore | null => lines?.[0]?.score ?? null;

/**
 * The move the eval bar's score belongs to — line 1's first move, by the
 * same rule as `topLineScore` and for the same reason. The best-move arrow
 * and the Engine tab's best-move row both read this, so neither can drift
 * back to picking the highest-scoring line.
 */
export const topLineMove = (
  lines: { moves: string[] }[] | undefined,
): string | null => lines?.[0]?.moves[0] ?? null;
