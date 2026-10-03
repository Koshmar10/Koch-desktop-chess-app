import type { LiveEngineSettings } from "../../../../api/bindings/LiveEngineSettings";
import { MAX_THREADS } from "../../../../api/settings";
import { NumberSetting } from "../../../../components/controls";
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
  settings: LiveEngineSettings;
  onChange: (next: LiveEngineSettings) => void;
}

// No search limit: the live engine searches until it's told to stop.
const EngineSettings = ({ settings, onChange }: EngineSettingsProps) => {
  const patch = (next: Partial<LiveEngineSettings>) =>
    onChange({ ...settings, ...next });

  return (
    <PanelSection title="Settings" divided>
      <p className="text-foreground/50 text-sm">
        * These are the settings the live engine works at to find variations
      </p>

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
