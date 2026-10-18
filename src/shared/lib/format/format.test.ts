import { describe, expect, it } from "vitest";
import { formatBytes, formatRelativeTime, pluralize, toIsoString } from "./format";

describe("formatBytes", () => {
  it("keeps small sizes in bytes", () => {
    expect(formatBytes(512)).toBe("512 B");
  });

  it("scales to the largest fitting unit", () => {
    expect(formatBytes(1536)).toBe("1.5 KB");
    expect(formatBytes(5 * 1024 * 1024)).toBe("5.0 MB");
  });
});

describe("toIsoString", () => {
  it("renders milliseconds as ISO date", () => {
    expect(toIsoString(0)).toBe("1970-01-01T00:00:00.000Z");
  });
});

describe("pluralize", () => {
  it("picks singular or plural", () => {
    expect(pluralize(1, "message")).toBe("1 message");
    expect(pluralize(0, "message")).toBe("0 messages");
    expect(pluralize(2, "reply", "replies")).toBe("2 replies");
  });
});

describe("formatRelativeTime", () => {
  const now = Date.UTC(2026, 9, 17, 12);

  it("describes recent times relatively and older ones by date", () => {
    expect(formatRelativeTime(now - 10_000, now)).toBe("now");
    expect(formatRelativeTime(now - 5 * 60_000, now)).toBe("5 minutes ago");
    expect(formatRelativeTime(now - 3 * 3_600_000, now)).toBe("3 hours ago");
    expect(formatRelativeTime(now - 26 * 3_600_000, now)).toBe("yesterday");
    expect(formatRelativeTime(now - 30 * 86_400_000, now)).not.toContain("ago");
  });
});
