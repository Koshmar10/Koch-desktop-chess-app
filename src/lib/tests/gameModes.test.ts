import { describe, expect, it } from "vitest";
import { modeForTimeControl, modeMinutesLabel } from "../gameModes";

const clock = (minutes: number) => ({
  initial_ms: minutes * 60_000,
  increment_ms: 0,
});

describe("modeForTimeControl", () => {
  it("names a preset's own clock as that preset", () => {
    expect(modeForTimeControl(clock(1))).toBe("Bullet");
    expect(modeForTimeControl(clock(3))).toBe("Blitz");
    expect(modeForTimeControl(clock(10))).toBe("Rapid");
    expect(modeForTimeControl(clock(30))).toBe("Classical");
  });

  // An imported game can have any clock; it goes in the fastest mode it fits.
  it("puts a clock between presets in the next one up", () => {
    expect(modeForTimeControl(clock(5))).toBe("Rapid");
  });

  it("calls anything longer than every preset Classical", () => {
    expect(modeForTimeControl(clock(90))).toBe("Classical");
  });
});

describe("modeMinutesLabel", () => {
  it("gives a mode's starting time in minutes", () => {
    expect(modeMinutesLabel("Blitz")).toBe("3 min");
  });
});
