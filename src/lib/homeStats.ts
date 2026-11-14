import type { GameSummary } from "../api/bindings/GameSummary";
import type { PieceColor } from "../api/bindings/PieceColor";
import { parseStoredDate } from "./time";

export const OTHER_OPENINGS_LABEL = "Other";

const NAMED_OPENING_COUNT = 3;
const PERCENT = 100;

export interface OpeningShare {
  name: string;
  count: number;
  percent: number;
}

// Your most-played openings as one colour, the rest folded into "Other".
// Games with no named opening or no human side are left out of both the
// slices and the total, so the shares always add up to the games shown.
export const openingShares = (
  games: GameSummary[],
  color: PieceColor,
): OpeningShare[] => {
  const counts = new Map<string, number>();
  for (const game of games) {
    if (game.human_color !== color || !game.opening_name) continue;
    counts.set(game.opening_name, (counts.get(game.opening_name) ?? 0) + 1);
  }
  const total = [...counts.values()].reduce((sum, count) => sum + count, 0);
  if (total === 0) return [];

  const ranked = [...counts].sort(([, a], [, b]) => b - a);
  const named = ranked.slice(0, NAMED_OPENING_COUNT);
  const otherCount = ranked
    .slice(NAMED_OPENING_COUNT)
    .reduce((sum, [, count]) => sum + count, 0);
  const slices =
    otherCount > 0 ? [...named, [OTHER_OPENINGS_LABEL, otherCount]] : named;

  return slices.map(([name, count]) => ({
    name: String(name),
    count: Number(count),
    percent: Math.round((Number(count) / total) * PERCENT),
  }));
};

// The newest games first, by the date they were played.
export const latestGames = (games: GameSummary[], count: number) =>
  [...games]
    .sort(
      (a, b) =>
        (parseStoredDate(b.date_played)?.getTime() ?? 0) -
        (parseStoredDate(a.date_played)?.getTime() ?? 0),
    )
    .slice(0, count);
