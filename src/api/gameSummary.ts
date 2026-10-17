import type { GameSummary } from "./bindings/GameSummary";

// SQLite's `datetime('now')` is UTC "YYYY-MM-DD HH:MM:SS" with no zone
// marker, so the Z has to be added or the browser reads it as local time
// and the date can land a day out. Shared by the history cards and the
// analyzer's game tab, which both print it.
export const formatDatePlayed = (datePlayed: string | null): string => {
  if (!datePlayed) return "";
  const date = new Date(`${datePlayed.replace(" ", "T")}Z`);
  if (isNaN(date.getTime())) return datePlayed;
  return date.toLocaleDateString(undefined, {
    year: "numeric",
    month: "short",
    day: "numeric",
  });
};

const WIN_SCORE_CARD_COLOR =
  "bg-emerald-500/15 text-emerald-600 dark:text-emerald-400 border border-emerald-500/30";
const LOSE_SCORE_CARD_COLOR =
  "bg-red-500/15 text-red-600 dark:text-red-400 border border-red-500/30";
const DRAW_SCORE_CARD_COLOR =
  "bg-muted/40 text-muted-foreground border border-border";

// The result's colors, from the human's side: green for a win, red for a
// loss. Only the colors — each caller sizes the chip for where it sits.
// Shared by the history cards and the analyzer's game picker, so a result
// reads the same in both. A null `human_color` (self-play, imports) is
// deliberately neutral, like a draw.
export const getScoreCardStyle = (game: GameSummary): string => {
  if (game.human_color === null || game.result === "1/2-1/2") {
    return DRAW_SCORE_CARD_COLOR;
  }
  const humanWon =
    (game.result === "1-0" && game.human_color === "white") ||
    (game.result === "0-1" && game.human_color === "black");
  return humanWon ? WIN_SCORE_CARD_COLOR : LOSE_SCORE_CARD_COLOR;
};
