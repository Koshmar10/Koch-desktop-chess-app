import type { AnalyzerEngineSettings } from "../../../../api/bindings/AnalyzerEngineSettings";
import {
  LABEL_TO_LIMIT_MODE,
  LIMIT_BOUNDS,
  LIMIT_DEFAULT_VALUE,
  LIMIT_MODE_LABEL,
  LIMIT_MODES,
  limitValueLabel,
} from "../../../../api/searchLimit";
import { MAX_THREADS } from "../../../../api/settings";
import { NumberSetting, SelectSetting } from "../../../../components/controls";
import PanelSection from "../PanelSection";

const MIN_LINES = 1;
const MAX_LINES = 5;
const MIN_HASH_MB = 16;
const MAX_HASH_MB = 4096;
const HASH_STEP_MB = 16;

// The Settings screen pairs each of these with a paragraph explaining the
// trade-off; here they're bare. This panel is for adjusting a value you
// already understand mid-analysis, not for learning what it does — the
// explanations stay where there's room to read them.
const Row = ({
  label,
  children,
}: {
  label: string;
  children: React.ReactNode;
}) => (
  <div className="flex items-center justify-between gap-2">
    <span className="text-xs text-foreground/55">{label}</span>
    {children}
  </div>
);

interface EngineSettingsProps {
  settings: AnalyzerEngineSettings;
  onChange: (next: AnalyzerEngineSettings) => void;
}

const EngineSettings = ({ settings, onChange }: EngineSettingsProps) => {
  const mode = settings.search_limit.mode;
  const bounds = LIMIT_BOUNDS[mode];

  const patch = (next: Partial<AnalyzerEngineSettings>) =>
    onChange({ ...settings, ...next });

  return (
    <PanelSection title="Settings" divided>
      <Row label="Limit">
        <SelectSetting
          value={LIMIT_MODE_LABEL[mode]}
          options={LIMIT_MODES.map((m) => LIMIT_MODE_LABEL[m])}
          onChange={(label) => {
            // Switching mode carries its own default rather than keeping
            // the old number: 20 plies of depth is sane, 20 milliseconds
            // of move time is not.
            const nextMode = LABEL_TO_LIMIT_MODE[label];
            patch({
              search_limit: {
                mode: nextMode,
                value: LIMIT_DEFAULT_VALUE[nextMode],
              },
            });
          }}
        />
      </Row>

      <Row label={limitValueLabel(mode)}>
        <NumberSetting
          value={settings.search_limit.value}
          onChange={(value) => patch({ search_limit: { mode, value } })}
          min={bounds.min}
          max={bounds.max}
          step={bounds.step}
        />
      </Row>

      <Row label="Lines">
        <NumberSetting
          value={settings.multi_pv}
          onChange={(multi_pv) => patch({ multi_pv })}
          min={MIN_LINES}
          max={MAX_LINES}
        />
      </Row>

      <Row label="Threads">
        <NumberSetting
          value={settings.threads}
          onChange={(threads) => patch({ threads })}
          min={1}
          max={MAX_THREADS}
        />
      </Row>

      <Row label="Hash (MB)">
        <NumberSetting
          value={settings.hash_mb}
          onChange={(hash_mb) => patch({ hash_mb })}
          min={MIN_HASH_MB}
          max={MAX_HASH_MB}
          step={HASH_STEP_MB}
        />
      </Row>
    </PanelSection>
  );
};

export default EngineSettings;
