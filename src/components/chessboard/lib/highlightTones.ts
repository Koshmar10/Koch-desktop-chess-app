import type { HighlightTone } from "./types";

// Square fills are faint: they only mark *which* pieces a finding
// involves, while the arrow over them carries the direction and the
// emphasis. Strong enough to pick out at a glance, weak enough that the
// square colour, the last-move tint and the piece itself all read through.
export const TONE_SQUARE_COLOR: Record<HighlightTone, string> = {
  info: "rgba(59, 130, 246, 0.26)",
  danger: "rgba(220, 38, 38, 0.26)",
  warning: "rgba(245, 158, 11, 0.26)",
  good: "rgba(16, 185, 129, 0.26)",
};

// Arrows are solid at the same hues: a stroke a fifth of a square wide
// covers far less ground than a filled square, so it needs the opacity
// the fills give up.
export const TONE_ARROW_COLOR: Record<HighlightTone, string> = {
  info: "#3b82f6",
  danger: "#dc2626",
  warning: "#f59e0b",
  good: "#10b981",
};
