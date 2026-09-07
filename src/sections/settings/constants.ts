import { AppSettings } from "../../api/bindings/AppSettings"
import { AnalyzerEngineSettings } from "../../api/bindings/AnalyzerEngineSettings"
import { PlayerEngineSettings } from "../../api/bindings/PlayerEngineSettings"

// Shared non-component values live here so the component files stay
// component-only (keeps React Fast Refresh working).

export const CONTROL_CLASS =
  "px-2 py-1 text-sm rounded-md bg-input/30 border border-border outline-none focus:border-primary"

export const TEXT_INPUT_CLASS = `${CONTROL_CLASS} w-full`

// Bounded by the machine so a number field can't ask Stockfish for more
// than the box has. Falls back to 1 where the browser doesn't report it.
export const MAX_THREADS =
  typeof navigator !== "undefined" && navigator.hardwareConcurrency
    ? navigator.hardwareConcurrency
    : 1

// Used before saved values load, and when a get_* command returns null
// (nothing saved yet).
export const DEFAULT_APP_SETTINGS: AppSettings = {
  koch_username: null,
  chessdotcom_username: null,
  openai_key: null,
}

export const DEFAULT_ANALYZER_SETTINGS: AnalyzerEngineSettings = {
  search_limit: { mode: "Depth", value: 20 },
  multi_pv: 1,
  threads: 1,
  hash_mb: 16,
}

export const DEFAULT_PLAYER_SETTINGS: PlayerEngineSettings = {
  threads: 1,
  hash_mb: 16,
  move_overhead_ms: 10,
  ponder: false,
}
