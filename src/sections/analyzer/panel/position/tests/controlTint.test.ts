import { describe, expect, it } from "vitest";
import type { PositionFindings } from "../../../../../api/bindings/PositionFindings";
import { MOCK_POSITION_FINDINGS } from "../../../mock";
import { controlCounts, controlOf, controlTints } from "../controlTint";

const SQUARE_COUNT = 64;

// A board with nothing covered except what each test sets.
const findingsWith = (
  squares: { index: number; white: number[]; black: number[] }[],
): PositionFindings => {
  const findings: PositionFindings = {
    ...MOCK_POSITION_FINDINGS,
    square_threats: Array.from({ length: SQUARE_COUNT }, () => 0),
    attackers: Array.from({ length: SQUARE_COUNT }, () => []),
    defenders: Array.from({ length: SQUARE_COUNT }, () => []),
  };
  for (const { index, white, black } of squares) {
    // White counts -1 and Black +1, as koch-engine's threat_map does.
    findings.square_threats[index] = black.length - white.length;
    findings.attackers[index] = white;
    findings.defenders[index] = black;
  }
  return findings;
};

describe("controlOf", () => {
  // The landmine: square_threats is negative for White, the reverse of
  // the eval bar. A swapped sign here paints the whole board for the
  // wrong side, so this is the test that has to fail first.
  it("reads a negative net as White's square", () => {
    const findings = findingsWith([{ index: 27, white: [30], black: [] }]);
    expect(controlOf(findings, 27)).toEqual({ side: "white", margin: 1 });
  });

  it("reads a positive net as Black's square", () => {
    const findings = findingsWith([{ index: 27, white: [], black: [1, 6] }]);
    expect(controlOf(findings, 27)).toEqual({ side: "black", margin: 2 });
  });

  it("tells a fought-over square from an untouched one", () => {
    const findings = findingsWith([{ index: 27, white: [30], black: [1] }]);
    expect(controlOf(findings, 27)).toEqual({
      side: "contested",
      margin: 0,
    });
    expect(controlOf(findings, 28)).toBeNull();
  });
});

describe("controlTints", () => {
  it("tints only covered squares, at their board coordinates", () => {
    // Index 27 is rank 3, file 3 — d5.
    const findings = findingsWith([{ index: 27, white: [30], black: [] }]);
    const tints = controlTints(findings);

    expect(tints).toHaveLength(1);
    expect(tints[0].square).toEqual({ rank: 3, file: 3 });
  });

  it("shades a bigger margin more strongly", () => {
    const findings = findingsWith([
      { index: 0, white: [30], black: [] },
      { index: 1, white: [30, 29, 27], black: [] },
    ]);
    const [thin, thick] = controlTints(findings);
    // "rgba(r, g, b, a)" → a
    const alpha = (color: string) => {
      const parts = color.split(",");
      return Number(parts[parts.length - 1].trim().slice(0, -1));
    };

    expect(alpha(thick.color)).toBeGreaterThan(alpha(thin.color));
  });
});

describe("controlCounts", () => {
  it("counts each side's squares and the contested ones", () => {
    const findings = findingsWith([
      { index: 0, white: [30], black: [] },
      { index: 1, white: [30], black: [] },
      { index: 2, white: [], black: [6] },
      { index: 3, white: [30], black: [6] },
    ]);
    expect(controlCounts(findings)).toEqual({
      white: 2,
      black: 1,
      contested: 1,
    });
  });
});
