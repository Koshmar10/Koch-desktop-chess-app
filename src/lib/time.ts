export const MS_PER_SECOND = 1000;
export const MS_PER_MINUTE = 60 * MS_PER_SECOND;

const SECONDS_PER_MINUTE = 60;
const MINUTES_PER_HOUR = 60;

const twoDigits = (n: number): string => n.toString().padStart(2, "0");

/** Time on a chess clock: "09:58". */
export const formatClock = (ms: number): string => {
  const totalSeconds = Math.floor(ms / MS_PER_SECOND);
  const minutes = Math.floor(totalSeconds / SECONDS_PER_MINUTE);
  return `${twoDigits(minutes)}:${twoDigits(totalSeconds % SECONDS_PER_MINUTE)}`;
};

/** A short span to the tenth of a second — how long a move took: "1.5s". */
export const formatSeconds = (ms: number): string =>
  `${(ms / MS_PER_SECOND).toFixed(1)}s`;

/** A long span in hours and minutes — time spent playing: "4h 32m". */
export const formatDuration = (seconds: number): string => {
  const totalMinutes = Math.round(seconds / SECONDS_PER_MINUTE);
  const hours = Math.floor(totalMinutes / MINUTES_PER_HOUR);
  const minutes = totalMinutes % MINUTES_PER_HOUR;
  if (hours === 0) return `${minutes}m`;
  if (minutes === 0) return `${hours}h`;
  return `${hours}h ${minutes}m`;
};

/**
 * A date the backend stored with SQLite's `datetime('now')`: UTC
 * "YYYY-MM-DD HH:MM:SS", with no zone marker. The Z has to be added, or
 * the browser reads it as local time and the date can land a day out.
 * Null for a missing or unreadable value.
 */
export const parseStoredDate = (stored: string | null): Date | null => {
  if (!stored) return null;
  const date = new Date(`${stored.replace(" ", "T")}Z`);
  return Number.isNaN(date.getTime()) ? null : date;
};
