import type { LiveEngineSettings } from "../../../../api/bindings/LiveEngineSettings";
import { NumberSetting } from "../../../../components/controls";
import PanelSection from "../../../../components/PanelSection";
import { ENGINE_LIMITS } from "../../../../lib/settings";

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
          {...ENGINE_LIMITS.lines}
        />
      </Row>

      <Row label="Threads">
        <NumberSetting
          value={settings.threads}
          onChange={(threads) => patch({ threads })}
          {...ENGINE_LIMITS.threads}
        />
      </Row>

      <Row label="Hash (MB)">
        <NumberSetting
          value={settings.hash_mb}
          onChange={(hash_mb) => patch({ hash_mb })}
          {...ENGINE_LIMITS.hashMb}
        />
      </Row>
    </PanelSection>
  );
};

export default EngineSettings;
