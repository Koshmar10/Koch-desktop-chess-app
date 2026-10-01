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
