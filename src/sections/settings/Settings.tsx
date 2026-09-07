import { useEffect, useState } from "react"
import { AppSettings } from "../../api/bindings/AppSettings"
import { AnalyzerEngineSettings } from "../../api/bindings/AnalyzerEngineSettings"
import { PlayerEngineSettings } from "../../api/bindings/PlayerEngineSettings"
import {
  getAppSettings,
  getAnalyzerEngineSettings,
  getPlayerEngineSettings,
  saveAppSettings,
  saveAnalyzerEngineSettings,
  savePlayerEngineSettings,
} from "../../api/settings"
import {
  DEFAULT_APP_SETTINGS,
  DEFAULT_ANALYZER_SETTINGS,
  DEFAULT_PLAYER_SETTINGS,
} from "./constants"
import { AppSettingsSection } from "./AppSettingsSection"
import { AnalyzerEngineSection } from "./AnalyzerEngineSection"
import { PlayerEngineSection } from "./PlayerEngineSection"

const Settings = () => {
  const [app, setApp] = useState<AppSettings | null>(null)
  const [analyzer, setAnalyzer] = useState<AnalyzerEngineSettings | null>(null)
  const [player, setPlayer] = useState<PlayerEngineSettings | null>(null)

  useEffect(() => {
    getAppSettings()
      .then((s) => setApp(s ?? DEFAULT_APP_SETTINGS))
      .catch(() => setApp(DEFAULT_APP_SETTINGS))
    getAnalyzerEngineSettings()
      .then((s) => setAnalyzer(s ?? DEFAULT_ANALYZER_SETTINGS))
      .catch(() => setAnalyzer(DEFAULT_ANALYZER_SETTINGS))
    getPlayerEngineSettings()
      .then((s) => setPlayer(s ?? DEFAULT_PLAYER_SETTINGS))
      .catch(() => setPlayer(DEFAULT_PLAYER_SETTINGS))
  }, [])

  // Wait for all three so a section never mounts with a placeholder it
  // would then blow away when the real saved values land.
  if (!app || !analyzer || !player) return null

  // Persist first, then adopt the draft as the new saved values (which
  // disables the section's Save/Cancel). A failed write leaves the
  // buttons enabled so the edit isn't silently lost.
  const persist =
    <T,>(save: (v: T) => Promise<void>, setSaved: (v: T) => void) =>
    (next: T) => {
      save(next)
        .then(() => setSaved(next))
        .catch((e) => console.error("failed to save settings:", e))
    }

  return (
    <div className="flex max-w-5xl flex-col gap-8 p-5">
      <AppSettingsSection
        saved={app}
        onSaved={persist(saveAppSettings, setApp)}
      />
      <AnalyzerEngineSection
        saved={analyzer}
        onSaved={persist(saveAnalyzerEngineSettings, setAnalyzer)}
      />
      <PlayerEngineSection
        saved={player}
        onSaved={persist(savePlayerEngineSettings, setPlayer)}
      />
    </div>
  )
}

export default Settings
