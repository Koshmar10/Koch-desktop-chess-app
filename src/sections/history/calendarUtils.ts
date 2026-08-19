export const MS_PER_DAY = 1000 * 60 * 60 * 24;

export function getDaysInCurrentYear(): number {
  const year = new Date().getFullYear();
  const start = new Date(year, 0, 1);
  const end = new Date(year + 1, 0, 1);
  const diff = end.getTime() - start.getTime();
  return diff / MS_PER_DAY;
}

export function dateToDayIndex(dateStr: string): number {
  const [year, month, day] = dateStr.split("-").map(Number);
  const dayMs = Date.UTC(year, month - 1, day);
  const startOfYearMs = Date.UTC(year, 0, 1);
  return Math.round((dayMs - startOfYearMs) / MS_PER_DAY);
}

export function formatDayLabel(dateStr: string): string {
  const [year, month, day] = dateStr.split("-").map(Number);
  const date = new Date(Date.UTC(year, month - 1, day));
  return new Intl.DateTimeFormat("en-US", {
    month: "short",
    day: "numeric",
    timeZone: "UTC",
  }).format(date);
}

export function formatDuration(seconds: number): string {
  const totalMinutes = Math.round(seconds / 60);
  const hours = Math.floor(totalMinutes / 60);
  const minutes = totalMinutes % 60;
  if (hours === 0) {
    return `${minutes}m`;
  }
  if (minutes === 0) {
    return `${hours}h`;
  }
  return `${hours}h ${minutes}m`;
}

const MIN_DURATION_SECONDS = 2 * 60; 
const MAX_DURATION_SECONDS = 4 * 60 * 60; 

export function scaleOpacity(
  duration: number,
  minOpacity = 0.1,
  maxOpacity = 1
): number {
  const clamped = Math.min(Math.max(duration, MIN_DURATION_SECONDS), MAX_DURATION_SECONDS);
  const ratio =
    (clamped - MIN_DURATION_SECONDS) / (MAX_DURATION_SECONDS - MIN_DURATION_SECONDS);
  return minOpacity + ratio * (maxOpacity - minOpacity);
}
