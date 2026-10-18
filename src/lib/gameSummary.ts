import type { GameResult } from "../api/bindings/GameResult";
import type { GameScore } from "../api/bindings/GameScore";
import type { TimeControl } from "../api/bindings/TimeControl";
import { MS_PER_MINUTE, MS_PER_SECOND, parseStoredDate } from "./time";

// The day a game was played, the way the history cards and the analyzer's
// game tab both print it. A value that won't read as a date is shown as
// it is rather than dropped.
export const formatDatePlayed = (datePlayed: string | null): string => {
  const date = parseStoredDate(datePlayed);
  if (!date) return datePlayed ?? "";
  return date.toLocaleDateString(undefined, {
    year: "numeric",
    month: "short",
    day: "numeric",
  });
};

// How a result is written wherever a person reads it — the PGN token, the
// notation every chess site and book uses.
const RESULT_TOKEN: Record<GameResult, string> = {
  WhiteWin: "1-0",
  BlackWin: "0-1",
  Draw: "1/2-1/2",
  Unfinished: "*",
};

export const resultToken = (result: GameResult): string => RESULT_TOKEN[result];

// The way players say a clock: minutes, then the increment in seconds —
// "10+0" for rapid, "3+2" for blitz with a two-second increment.
export const formatTimeControl = ({
  initial_ms,
  increment_ms,
}: TimeControl): string =>
  `${initial_ms / MS_PER_MINUTE}+${increment_ms / MS_PER_SECOND}`;

const SCORE_CARD_COLOR: Record<GameScore, string> = {
  Win: "bg-emerald-500/15 text-emerald-600 dark:text-emerald-400 border border-emerald-500/30",
  Loss: "bg-red-500/15 text-red-600 dark:text-red-400 border border-red-500/30",
  Draw: "bg-muted/40 text-muted-foreground border border-border",
};

// The result's colors, from the human's side: green for a win, red for a
// loss. Only the colors — each caller sizes the chip for where it sits.
// Shared by the history cards and the analyzer's game picker, so a result
// reads the same in both. A game with no human side to score it from
// (self-play, an import nobody has picked a side for) is deliberately
// neutral, like a draw.
export const getScoreCardStyle = (score: GameScore | null): string =>
  SCORE_CARD_COLOR[score ?? "Draw"];
