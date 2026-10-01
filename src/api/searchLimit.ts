import type { SearchLimitMode } from "./bindings/SearchLimitMode";

// Shape and bounds for the analyzer's stop condition, shared by the two
// screens that edit it — the Settings page and the Analyzer's engine tab.
// The long explanatory hints stay with the Settings page: it has room to
// show them and the compact panel doesn't.

export const LIMIT_MODES: SearchLimitMode[] = ["Depth", "MoveTime", "Nodes"];

// The binding's variant names ↔ what a select shows.
export const LIMIT_MODE_LABEL: Record<SearchLimitMode, string> = {
  Depth: "Depth",
  MoveTime: "Move time",
  Nodes: "Nodes",
};

export const LABEL_TO_LIMIT_MODE: Record<string, SearchLimitMode> = {
  Depth: "Depth",
  "Move time": "MoveTime",
  Nodes: "Nodes",
};

// UI-only bounds per limit mode: depth in plies, move time in milliseconds,
// nodes as a raw count — what a number field allows before the value
// reaches the backend that runs the search.
export const LIMIT_BOUNDS: Record<
  SearchLimitMode,
  { min: number; max: number; step: number }
> = {
  Depth: { min: 1, max: 60, step: 1 },
  MoveTime: { min: 100, max: 60_000, step: 100 },
  Nodes: { min: 10_000, max: 100_000_000, step: 10_000 },
};

export const LIMIT_DEFAULT_VALUE: Record<SearchLimitMode, number> = {
  Depth: 20,
  MoveTime: 2_000,
  Nodes: 1_000_000,
};

/** The row label for the limit's value, which changes with the mode. */
export const limitValueLabel = (mode: SearchLimitMode): string =>
  mode === "MoveTime" ? "Move time (ms)" : LIMIT_MODE_LABEL[mode];
