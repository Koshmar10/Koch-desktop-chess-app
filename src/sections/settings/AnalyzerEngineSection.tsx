import { AnalyzerEngineSettings } from "../../api/bindings/AnalyzerEngineSettings";
import { SearchLimitMode } from "../../api/bindings/SearchLimitMode";
import { Section, SettingRow } from "./Section";
import { NumberSetting, SelectSetting } from "../../components/controls";
import { MAX_THREADS } from "../../api/settings";
import {
  LABEL_TO_LIMIT_MODE,
  LIMIT_BOUNDS,
  LIMIT_DEFAULT_VALUE,
  LIMIT_MODE_LABEL,
  LIMIT_MODES,
  limitValueLabel,
} from "../../api/searchLimit";
import { isDirty, useDraft, type SectionState } from "./draft";

const LIMIT_HINT: Record<SearchLimitMode, string> = {
  Depth:
    "Half-moves Stockfish looks ahead at each position. 20 is plenty for reviewing a game; every extra few plies can roughly double the run time for an accuracy gain you won't notice.",
  MoveTime:
    "Milliseconds spent on each position. At 2000 ms a 40-move game takes over a minute to analyse; 10000 ms pushes a single game past six minutes.",
  Nodes:
    "Search-tree positions visited per move. Machine-independent, so a run costs the same effort on any computer — but the wall-clock time still grows with the count.",
};

export const AnalyzerEngineSection = ({
  saved,
  onSaved,
}: SectionState<AnalyzerEngineSettings>) => {
  const [draft, setDraft] = useDraft(saved);

  const patch = (next: Partial<AnalyzerEngineSettings>) =>
    setDraft((current) => ({ ...current, ...next }));

  const mode = draft.search_limit.mode;
  const bounds = LIMIT_BOUNDS[mode];

  return (
    <Section
      title="Analyzer Engine"
      subtitle="How the engine reviews finished games. Higher settings mean slower, more thorough passes."
      dirty={isDirty(saved, draft)}
      onSave={() => onSaved(draft)}
      onCancel={() => setDraft(saved)}
    >
      <SettingRow
        label="Search limit"
        hint="How the analyzer decides it's done with a position: a fixed Depth, a fixed Move time, or a number of Nodes. Depth is the most consistent, Move time the most predictable in real seconds."
      >
        <SelectSetting
          value={LIMIT_MODE_LABEL[mode]}
          options={LIMIT_MODES.map((m) => LIMIT_MODE_LABEL[m])}
          onChange={(label) => {
            const nextMode = LABEL_TO_LIMIT_MODE[label];
            patch({
              search_limit: {
                mode: nextMode,
                value: LIMIT_DEFAULT_VALUE[nextMode],
              },
            });
          }}
        />
      </SettingRow>
      <SettingRow label={limitValueLabel(mode)} hint={LIMIT_HINT[mode]}>
        <NumberSetting
          value={draft.search_limit.value}
          onChange={(v) => patch({ search_limit: { mode, value: v } })}
          min={bounds.min}
          max={bounds.max}
          step={bounds.step}
        />
      </SettingRow>
      <SettingRow
        label="Lines (MultiPV)"
        hint="How many alternative best moves to score per position. 1 is fastest; raising it to 5 makes Stockfish fully evaluate every candidate and can slow analysis 2–3×."
      >
        <NumberSetting
          value={draft.multi_pv}
          onChange={(v) => patch({ multi_pv: v })}
          min={1}
          max={5}
        />
      </SettingRow>
      <SettingRow
        label="Threads"
        hint={`CPU cores the analyzer searches with — more cores, faster passes. Going above your ${MAX_THREADS} physical cores won't help and just competes with the rest of the app.`}
      >
        <NumberSetting
          value={draft.threads}
          onChange={(v) => patch({ threads: v })}
          min={1}
          max={MAX_THREADS}
        />
      </SettingRow>
      <SettingRow
        label="Hash (MB)"
        hint="Memory for Stockfish's transposition table. 16 is fine for short searches; setting 4096 reserves 4 GB of RAM for no gain unless each position runs for many seconds."
      >
        <NumberSetting
          value={draft.hash_mb}
          onChange={(v) => patch({ hash_mb: v })}
          min={16}
          max={4096}
          step={16}
        />
      </SettingRow>
    </Section>
  );
};
