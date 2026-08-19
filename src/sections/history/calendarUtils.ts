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

export function scaleOpacity(
  duration: number,
  minDuration: number,
  maxDuration: number,
  minOpacity = 0.1,
  maxOpacity = 1
): number {
  if (maxDuration === minDuration) {
    return maxOpacity;
  }
  const ratio = (duration - minDuration) / (maxDuration - minDuration);
  return minOpacity + ratio * (maxOpacity - minOpacity);
}
