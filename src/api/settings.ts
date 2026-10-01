import { invoke } from "@tauri-apps/api/core";
import { AppSettings } from "./bindings/AppSettings";
import { AnalyzerEngineSettings } from "./bindings/AnalyzerEngineSettings";
import { PlayerEngineSettings } from "./bindings/PlayerEngineSettings";

// Each returns null when nothing has been saved yet — the caller falls
// back to its own defaults.
export const getAppSettings = (): Promise<AppSettings | null> =>
  invoke<AppSettings | null>("get_app_settings");

export const getAnalyzerEngineSettings =
  (): Promise<AnalyzerEngineSettings | null> =>
    invoke<AnalyzerEngineSettings | null>("get_analyzer_engine_settings");

export const getPlayerEngineSettings =
  (): Promise<PlayerEngineSettings | null> =>
    invoke<PlayerEngineSettings | null>("get_player_engine_settings");

// Each appends a new snapshot row (settings tables are append-only).
export const saveAppSettings = (settings: AppSettings): Promise<void> =>
  invoke("save_app_settings", { settings });

export const saveAnalyzerEngineSettings = (
  settings: AnalyzerEngineSettings,
): Promise<void> => invoke("save_analyzer_engine_settings", { settings });

export const savePlayerEngineSettings = (
  settings: PlayerEngineSettings,
): Promise<void> => invoke("save_player_engine_settings", { settings });

// Bounded by the machine so a number field can't ask Stockfish for more
// than the box has. Falls back to 1 where the browser doesn't report it.
export const MAX_THREADS =
  typeof navigator !== "undefined" && navigator.hardwareConcurrency
    ? navigator.hardwareConcurrency
    : 1;

// Used before saved values load, and when a get_* command returns null
// (nothing saved yet). These live beside the commands rather than in a
// screen folder because two screens now read them — the Settings page and
// the Analyzer's engine tab.
export const DEFAULT_APP_SETTINGS: AppSettings = {
  koch_username: null,
  chessdotcom_username: null,
  openai_key: null,
};

export const DEFAULT_ANALYZER_SETTINGS: AnalyzerEngineSettings = {
  search_limit: { mode: "Depth", value: 20 },
  multi_pv: 1,
  threads: 1,
  hash_mb: 16,
};

export const DEFAULT_PLAYER_SETTINGS: PlayerEngineSettings = {
  threads: 1,
  hash_mb: 16,
  move_overhead_ms: 10,
  ponder: false,
};
