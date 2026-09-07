import { invoke } from "@tauri-apps/api/core"
import { AppSettings } from "./bindings/AppSettings"
import { AnalyzerEngineSettings } from "./bindings/AnalyzerEngineSettings"
import { PlayerEngineSettings } from "./bindings/PlayerEngineSettings"

// Each returns null when nothing has been saved yet — the caller falls
// back to its own defaults.
export const getAppSettings = (): Promise<AppSettings | null> =>
  invoke<AppSettings | null>("get_app_settings")

export const getAnalyzerEngineSettings = (): Promise<AnalyzerEngineSettings | null> =>
  invoke<AnalyzerEngineSettings | null>("get_analyzer_engine_settings")

export const getPlayerEngineSettings = (): Promise<PlayerEngineSettings | null> =>
  invoke<PlayerEngineSettings | null>("get_player_engine_settings")

// Each appends a new snapshot row (settings tables are append-only).
export const saveAppSettings = (settings: AppSettings): Promise<void> =>
  invoke("save_app_settings", { settings })

export const saveAnalyzerEngineSettings = (
  settings: AnalyzerEngineSettings,
): Promise<void> => invoke("save_analyzer_engine_settings", { settings })

export const savePlayerEngineSettings = (
  settings: PlayerEngineSettings,
): Promise<void> => invoke("save_player_engine_settings", { settings })
