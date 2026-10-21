// A puzzle's rating in hundreds is what it's worth: a 1500 puzzle, 15.
const RATING_PER_POINT = 100;
// Each puzzle already in the clean run adds a tenth to the next one's
// worth...
const STREAK_BONUS_PER_PUZZLE = 0.1;
// ...up to double, ten puzzles in. Past that the streak still counts, but
// pays no more.
export const MAX_STREAK_MULTIPLIER = 2;

/** What the clean run so far multiplies a solve by. */
export const streakMultiplier = (streak: number): number =>
  Math.min(1 + streak * STREAK_BONUS_PER_PUZZLE, MAX_STREAK_MULTIPLIER);

/**
 * What a solved puzzle is worth: its rating in hundreds, halved for every
 * mistake, times the bonus for the clean run it extends.
 *
 * `streak` is the run *before* this puzzle. So the first clean solve
 * earns no bonus yet, and a puzzle with mistakes earns none at all: its
 * first mistake already ended the run.
 */
export const puzzlePoints = (
  rating: number,
  mistakes: number,
  streak: number,
): number => {
  const base = Math.round(rating / RATING_PER_POINT);
  const afterMistakes = Math.floor(base / 2 ** mistakes);
  return Math.round(afterMistakes * streakMultiplier(streak));
};
