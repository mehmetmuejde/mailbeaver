const BYTES_PER_KIB = 1024;
const SIZE_UNITS = ["B", "KB", "MB", "GB"] as const;

const dateTimeFormat = new Intl.DateTimeFormat(undefined, {
  dateStyle: "medium",
  timeStyle: "short",
});

export function formatBytes(bytes: number): string {
  let value = bytes;
  let unit = 0;
  while (value >= BYTES_PER_KIB && unit < SIZE_UNITS.length - 1) {
    value /= BYTES_PER_KIB;
    unit += 1;
  }
  const digits = unit === 0 ? 0 : 1;
  return `${value.toFixed(digits)} ${SIZE_UNITS[unit]}`;
}

export function formatDateTime(timestampMs: number): string {
  return dateTimeFormat.format(new Date(timestampMs));
}

export function toIsoString(timestampMs: number): string {
  return new Date(timestampMs).toISOString();
}

/** `1 message`, `3 messages`; pass the plural when it is not just an added "s". */
export function pluralize(count: number, singular: string, plural = `${singular}s`): string {
  return `${count} ${count === 1 ? singular : plural}`;
}

const MINUTE_MS = 60_000;
const HOUR_MS = 60 * MINUTE_MS;
const DAY_MS = 24 * HOUR_MS;
const RELATIVE_DAYS = 7;

const relativeFormat = new Intl.RelativeTimeFormat("en", { numeric: "auto" });
const shortDateFormat = new Intl.DateTimeFormat(undefined, { dateStyle: "medium" });

/** `now`, `5 minutes ago`, `yesterday`; older than a week the date itself. */
export function formatRelativeTime(timestampMs: number, nowMs: number = Date.now()): string {
  const elapsed = nowMs - timestampMs;
  if (elapsed < MINUTE_MS) {
    return "now";
  }
  if (elapsed < HOUR_MS) {
    return relativeFormat.format(-Math.floor(elapsed / MINUTE_MS), "minute");
  }
  if (elapsed < DAY_MS) {
    return relativeFormat.format(-Math.floor(elapsed / HOUR_MS), "hour");
  }
  if (elapsed < RELATIVE_DAYS * DAY_MS) {
    return relativeFormat.format(-Math.floor(elapsed / DAY_MS), "day");
  }
  return shortDateFormat.format(new Date(timestampMs));
}
