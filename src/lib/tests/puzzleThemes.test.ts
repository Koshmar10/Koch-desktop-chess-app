import { describe, expect, it } from "vitest";
import { formatPuzzleTheme } from "../puzzleThemes";

describe("formatPuzzleTheme", () => {
  it("splits camelCase into a sentence-case phrase", () => {
    expect(formatPuzzleTheme("discoveredAttack")).toBe("Discovered attack");
  });

  it("keeps a single word capitalised", () => {
    expect(formatPuzzleTheme("fork")).toBe("Fork");
  });

  it("separates a trailing number", () => {
    expect(formatPuzzleTheme("mateIn2")).toBe("Mate in 2");
  });
});
