import { describe, expect, it } from "vitest";
import {
  formatClock,
  formatDuration,
  formatSeconds,
  parseStoredDate,
} from "../time";

describe("formatClock", () => {
  it("pads minutes and seconds to two digits", () => {
    expect(formatClock(598_000)).toBe("09:58");
    expect(formatClock(0)).toBe("00:00");
  });

  // A clock never shows a second it hasn't fully used.
  it("drops part of a second rather than rounding it up", () => {
    expect(formatClock(59_999)).toBe("00:59");
  });
});

describe("formatSeconds", () => {
  it("gives a tenth of a second", () => {
    expect(formatSeconds(1_540)).toBe("1.5s");
  });
});

describe("formatDuration", () => {
  it("says hours and minutes, leaving out a part that's zero", () => {
    expect(formatDuration(4 * 3600 + 32 * 60)).toBe("4h 32m");
    expect(formatDuration(45 * 60)).toBe("45m");
    expect(formatDuration(2 * 3600)).toBe("2h");
  });
});

describe("parseStoredDate", () => {
  // The stored value has no zone marker but is UTC; read as local time it
  // could land on the wrong day.
  it("reads a stored timestamp as UTC", () => {
    expect(parseStoredDate("2026-10-17 23:30:00")?.toISOString()).toBe(
      "2026-10-17T23:30:00.000Z",
    );
  });

  it("is null for a missing or unreadable value", () => {
    expect(parseStoredDate(null)).toBeNull();
    expect(parseStoredDate("not a date")).toBeNull();
  });
});
