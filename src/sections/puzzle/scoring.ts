const RATING_PER_POINT = 100;
const STREAK_BONUS_PER_PUZZLE = 0.1;
export const MAX_STREAK_MULTIPLIER = 2;

export const streakMultiplier = (streak: number): number =>
  Math.min(1 + streak * STREAK_BONUS_PER_PUZZLE, MAX_STREAK_MULTIPLIER);

// `streak` is the run before this puzzle, so a first clean solve earns no
// bonus, and a puzzle with mistakes, whose first mistake ended the run,
// earns none either.
export const puzzlePoints = (
  rating: number,
  mistakes: number,
  streak: number,
): number => {
  const base = Math.round(rating / RATING_PER_POINT);
  const afterMistakes = Math.floor(base / 2 ** mistakes);
  return Math.round(afterMistakes * streakMultiplier(streak));
};
