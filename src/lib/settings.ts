import type { AnalyzerEngineSettings } from "../api/bindings/AnalyzerEngineSettings";
import type { AppSettings } from "../api/bindings/AppSettings";
import type { LiveEngineSettings } from "../api/bindings/LiveEngineSettings";
import type { PlayerEngineSettings } from "../api/bindings/PlayerEngineSettings";
import { LIMIT_DEFAULT_VALUE } from "./searchLimit";

// Bounded by the machine so a number field can't ask Stockfish for more
// than the box has. Falls back to 1 where the browser doesn't report it.
export const MAX_THREADS =
  typeof navigator !== "undefined" && navigator.hardwareConcurrency
    ? navigator.hardwareConcurrency
    : 1;

interface NumberLimits {
  min: number;
  max: number;
  step: number;
}

/**
 * What each engine number field allows, before a value ever reaches
 * Stockfish. Shared by the Settings page and the analyzer's Engine tab,
 * which edit the same options and used to set their own limits for them.
 * Shaped to spread straight onto a `NumberSetting`.
 */
export const ENGINE_LIMITS = {
  lines: { min: 1, max: 5, step: 1 },
  threads: { min: 1, max: MAX_THREADS, step: 1 },
  hashMb: { min: 16, max: 4096, step: 16 },
  // The opponent thinks for about a second a move, which never fills more.
  playerHashMb: { min: 16, max: 1024, step: 16 },
  moveOverheadMs: { min: 10, max: 1000, step: 10 },
} satisfies Record<string, NumberLimits>;

// Stockfish's own defaults — what a fresh engine runs with, and so what
// the live engine shows until its session reports its real values.
export const DEFAULT_LIVE_ENGINE_SETTINGS: LiveEngineSettings = {
  multi_pv: 1,
  threads: 1,
  hash_mb: 16,
};

// The rest are used before saved values load, and when a get_* command
// returns null — nothing saved yet.

export const DEFAULT_APP_SETTINGS: AppSettings = {
  koch_username: null,
  chessdotcom_username: null,
  openai_key: null,
};

export const DEFAULT_ANALYZER_SETTINGS: AnalyzerEngineSettings = {
  ...DEFAULT_LIVE_ENGINE_SETTINGS,
  search_limit: { mode: "Depth", value: LIMIT_DEFAULT_VALUE.Depth },
};

export const DEFAULT_PLAYER_SETTINGS: PlayerEngineSettings = {
  threads: DEFAULT_LIVE_ENGINE_SETTINGS.threads,
  hash_mb: DEFAULT_LIVE_ENGINE_SETTINGS.hash_mb,
  move_overhead_ms: ENGINE_LIMITS.moveOverheadMs.min,
  ponder: false,
};
