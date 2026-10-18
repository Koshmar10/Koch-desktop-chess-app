import type { PlayerEngineSettings } from "../../api/bindings/PlayerEngineSettings";
import { Section, SettingRow } from "./Section";
import { NumberSetting, ToggleSetting } from "../../components/controls";
import { ENGINE_LIMITS, MAX_THREADS } from "../../lib/settings";
import { isDirty, useDraft, type SectionState } from "./draft";

const PlayerEngineSection = ({
  saved,
  onSaved,
}: SectionState<PlayerEngineSettings>) => {
  const [draft, setDraft] = useDraft(saved);

  const patch = (next: Partial<PlayerEngineSettings>) =>
    setDraft((current) => ({ ...current, ...next }));

  return (
    <Section
      title="Player Engine"
      subtitle="Resources for the opponent you play against. Performance only — difficulty lives in the game setup."
      dirty={isDirty(saved, draft)}
      onSave={() => onSaved(draft)}
      onCancel={() => setDraft(saved)}
    >
      <SettingRow
        label="Threads"
        hint={`CPU cores your opponent engine uses to pick a move. More cores make it decide faster within its time budget, not fundamentally stronger. Above your ${MAX_THREADS} physical cores it only fights the UI for CPU.`}
      >
        <NumberSetting
          value={draft.threads}
          onChange={(v) => patch({ threads: v })}
          {...ENGINE_LIMITS.threads}
        />
      </SettingRow>
      <SettingRow
        label="Hash (MB)"
        hint="Memory for the opponent engine's transposition table. A one-second-per-move search barely fills 16 MB, so larger values here mostly just reserve RAM you won't use."
      >
        <NumberSetting
          value={draft.hash_mb}
          onChange={(v) => patch({ hash_mb: v })}
          {...ENGINE_LIMITS.playerHashMb}
        />
      </SettingRow>
      <SettingRow
        label="Move overhead (ms)"
        hint="A margin shaved off the engine's clock to absorb communication lag so it doesn't lose on time. 10 ms is right for a local engine; only raise it if you see the engine flagging."
      >
        <NumberSetting
          value={draft.move_overhead_ms}
          onChange={(v) => patch({ move_overhead_ms: v })}
          {...ENGINE_LIMITS.moveOverheadMs}
        />
      </SettingRow>
      <SettingRow
        label="Ponder"
        hint="Lets the opponent engine keep calculating during your turn, on otherwise idle CPU. Slightly stronger play, at the cost of constant background load and more battery drain on a laptop."
      >
        <ToggleSetting
          checked={draft.ponder}
          onChange={(v) => patch({ ponder: v })}
        />
      </SettingRow>
    </Section>
  );
};

export default PlayerEngineSection;
