export type Period = "Last Month" | "Last 3 Months" | "All Time";
export const PERIODS: Period[] = ["Last Month", "Last 3 Months", "All Time"];

const PERIOD_MONTHS: Record<Period, number | null> = {
  "Last Month": 1,
  "Last 3 Months": 3,
  "All Time": null,
};

/** The first day a period covers, as "YYYY-MM-DD"; none for all time. */
export const periodStart = (period: Period): string | undefined => {
  const months = PERIOD_MONTHS[period];
  if (months === null) return undefined;
  const start = new Date();
  start.setMonth(start.getMonth() - months);
  return start.toISOString().slice(0, 10);
};
