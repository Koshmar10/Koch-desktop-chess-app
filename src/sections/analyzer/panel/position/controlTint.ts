import type { PositionFindings } from "../../../../api/bindings/PositionFindings";
import { BOARD_SIZE } from "../../../../components/chessboard/lib/constants";
import type { SquareMark } from "../../../../components/chessboard/lib/types";

export type ControlSide = "white" | "black" | "contested";

// One hue per side, plus purple for squares both sides cover equally —
// the same three the old app's influence tint used, so anyone who knew
// that screen reads this one without relearning it.
const SIDE_RGB: Record<ControlSide, string> = {
  white: "37, 99, 235",
  black: "220, 38, 38",
  contested: "128, 0, 128",
};

/** A side's colour at full strength — for legends, not the board itself. */
export const controlColor = (side: ControlSide): string =>
  `rgb(${SIDE_RGB[side]})`;

// A one-attacker margin is barely tinted and anything from three up is at
// full strength — beyond that, more attackers on a square stops telling
// you anything you can see.
const MIN_ALPHA = 0.12;
const MAX_ALPHA = 0.4;
const FULL_STRENGTH_MARGIN = 3;

/**
 * Who controls a square, from `PositionFindings`.
 *
 * ⚠️ `square_threats` runs **negative where White has the pressure** —
 * the reverse of the eval bar's "+ favours White". That inversion is
 * read in exactly one place, here; nothing downstream sees a raw value.
 *
 * Returns null for a square nobody covers. A net of 0 is ambiguous on its
 * own — untouched, or evenly fought over — so `attackers`/`defenders`
 * (White's and Black's coverers respectively) decide which.
 */
export const controlOf = (
  findings: PositionFindings,
  index: number,
): { side: ControlSide; margin: number } | null => {
  const net = findings.square_threats[index];
  if (net < 0) return { side: "white", margin: -net };
  if (net > 0) return { side: "black", margin: net };

  const covered =
    findings.attackers[index].length > 0 ||
    findings.defenders[index].length > 0;
  return covered ? { side: "contested", margin: 0 } : null;
};

const alphaFor = (margin: number): number => {
  const strength =
    Math.min(margin, FULL_STRENGTH_MARGIN) / FULL_STRENGTH_MARGIN;
  return MIN_ALPHA + strength * (MAX_ALPHA - MIN_ALPHA);
};

/** Every covered square, coloured by who holds it and by how much. */
export const controlTints = (findings: PositionFindings): SquareMark[] =>
  findings.square_threats.flatMap((_, index) => {
    const control = controlOf(findings, index);
    if (!control) return [];
    // Contested squares are a tie, not a margin — they get the floor, so
    // they read as "fought over" without outshouting a clear advantage.
    const alpha =
      control.side === "contested" ? MIN_ALPHA : alphaFor(control.margin);
    return [
      {
        square: {
          rank: Math.floor(index / BOARD_SIZE),
          file: index % BOARD_SIZE,
        },
        color: `rgba(${SIDE_RGB[control.side]}, ${alpha.toFixed(3)})`,
      },
    ];
  });

/** How many squares each side holds, for the section's summary. */
export const controlCounts = (
  findings: PositionFindings,
): Record<ControlSide, number> => {
  const counts: Record<ControlSide, number> = {
    white: 0,
    black: 0,
    contested: 0,
  };
  findings.square_threats.forEach((_, index) => {
    const control = controlOf(findings, index);
    if (control) counts[control.side] += 1;
  });
  return counts;
};
