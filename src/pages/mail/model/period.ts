import type { Period } from "./mail-search";

const PERIOD_DAYS: Record<Period, number> = { today: 1, "7d": 7, "30d": 30 };

/**
 * Start of a period in local time, in Unix milliseconds: midnight of the first day.
 * Whole days keep the value stable for a day, so list queries stay cached.
 */
export function periodStart(period: Period, now: Date = new Date()): number {
  const start = new Date(now);
  start.setHours(0, 0, 0, 0);
  start.setDate(start.getDate() - (PERIOD_DAYS[period] - 1));
  return start.getTime();
}
