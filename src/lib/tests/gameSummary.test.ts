import { describe, expect, it } from "vitest";
import {
  formatTimeControl,
  getScoreCardStyle,
  resultToken,
} from "../gameSummary";

describe("resultToken", () => {
  it("writes each result the way PGN and players do", () => {
    expect(resultToken("WhiteWin")).toBe("1-0");
    expect(resultToken("BlackWin")).toBe("0-1");
    expect(resultToken("Draw")).toBe("1/2-1/2");
    expect(resultToken("Unfinished")).toBe("*");
  });
});

describe("formatTimeControl", () => {
  it("gives minutes, then the increment in seconds", () => {
    expect(formatTimeControl({ initial_ms: 600_000, increment_ms: 0 })).toBe(
      "10+0",
    );
    expect(
      formatTimeControl({ initial_ms: 180_000, increment_ms: 2_000 }),
    ).toBe("3+2");
  });
});

describe("getScoreCardStyle", () => {
  it("colours a win and a loss differently", () => {
    expect(getScoreCardStyle("Win")).not.toBe(getScoreCardStyle("Loss"));
  });

  // A game with no human side has nobody to win or lose it.
  it("shows a game with no score the way it shows a draw", () => {
    expect(getScoreCardStyle(null)).toBe(getScoreCardStyle("Draw"));
  });
});
