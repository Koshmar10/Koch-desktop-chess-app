import { describe, expect, it } from "vitest";
import { MOVE_QUALITY_ORDER, countByQuality } from "../moveQuality";

describe("countByQuality", () => {
  it("counts each grade", () => {
    const counts = countByQuality(["Good", "Blunder", "Good"]);

    expect(counts.Good).toBe(2);
    expect(counts.Blunder).toBe(1);
  });

  // A breakdown always shows the same rows, so a grade no move got is 0,
  // not missing.
  it("has every grade, at zero when no move got it", () => {
    const counts = countByQuality([]);

    for (const quality of MOVE_QUALITY_ORDER) {
      expect(counts[quality]).toBe(0);
    }
  });
});
