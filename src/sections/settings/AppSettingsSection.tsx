import { Moon, Sun } from "lucide-react"
import { useAppContext } from "../../app/AppContext"
import { AppSettings } from "../../api/bindings/AppSettings"
import { Section, SettingRow } from "./Section"
import { SecretInput } from "./controls"
import { TEXT_INPUT_CLASS } from "./constants"
import { isDirty, useDraft, type SectionState } from "./draft"

export const AppSettingsSection = ({
  saved,
  onSaved,
}: SectionState<AppSettings>) => {
  const { theme, toggleTheme } = useAppContext()
  const [draft, setDraft] = useDraft(saved)

  const patch = (next: Partial<AppSettings>) =>
    setDraft((current) => ({ ...current, ...next }))

  return (
    <Section
      title="App Settings"
      subtitle="Preferences for the app itself."
      dirty={isDirty(saved, draft)}
      onSave={() => onSaved(draft)}
      onCancel={() => setDraft(saved)}
    >
      {/* Theme is applied and persisted live by AppContext — not part of
          this section's draft, so Save/Cancel don't touch it. */}
      <SettingRow
        label="Change Theme"
        hint="Light or dark colour scheme for the whole app. Applied instantly and remembered the next time you launch."
      >
        <button
          type="button"
          onClick={toggleTheme}
          className="flex items-center gap-2 rounded-md border border-border px-2 py-1 text-sm hover:bg-border/60"
        >
          {theme === "dark" ? <Moon size={14} /> : <Sun size={14} />}
          {theme === "dark" ? "Dark" : "Light"}
        </button>
      </SettingRow>
      <SettingRow
        label="Koch Username"
        hint="The name recorded as your side in games you play inside Koch. Cosmetic — stored with each saved game and shown in history."
      >
        <input
          type="text"
          value={draft.koch_username ?? ""}
          onChange={(e) => patch({ koch_username: e.target.value || null })}
          placeholder="username"
          autoComplete="off"
          spellCheck={false}
          className={TEXT_INPUT_CLASS}
        />
      </SettingRow>
      <SettingRow
        label="Chess.com Username"
        hint="Used to pull your games from chess.com when importing. A wrong name isn't an error — the import just finds nothing."
      >
        <input
          type="text"
          value={draft.chessdotcom_username ?? ""}
          onChange={(e) =>
            patch({ chessdotcom_username: e.target.value || null })
          }
          placeholder="username"
          autoComplete="off"
          spellCheck={false}
          className={TEXT_INPUT_CLASS}
        />
      </SettingRow>
      <SettingRow
        label="OpenAI Key"
        hint="Enables the position assistant. Kept in your OS keychain, never written to a file. Leave it blank and the assistant stays disabled."
      >
        <SecretInput
          value={draft.openai_key ?? ""}
          onChange={(v) => patch({ openai_key: v || null })}
          placeholder="sk-..."
        />
      </SettingRow>
    </Section>
  )
}
