import type { PlacedPiece } from "../../../../components/chessboard/lib/types";
import EngineLines from "./EngineLines";
import EngineOverlays from "./EngineOverlays";
import EngineSettings from "./EngineSettings";
import { useLiveEngineSettings } from "./useLiveEngineSettings";
import type { EngineSnapshot, EngineStatus } from "../../types";

interface EngineTabProps {
  snapshot: EngineSnapshot | null;
  status: EngineStatus;
  onToggleStatus: () => void;
  // New engine settings stop the search to apply; this picks it back up.
  onSettingsApplied: () => void;
  bestMove: string | null;
  threatMove: string | null;
  pieces: PlacedPiece[];
}

// Settings and the board switches first at their natural height, then the
// lines taking whatever's left and scrolling on their own — so a growing
// line list never pushes the controls around, and the controls sit at a
// stable position you can reach for without looking.
const EngineTab = ({
  snapshot,
  status,
  onToggleStatus,
  onSettingsApplied,
  bestMove,
  threatMove,
  pieces,
}: EngineTabProps) => {
  const [settings, setSettings] = useLiveEngineSettings(onSettingsApplied);

  return (
    <div className="flex min-h-0 flex-1 flex-col gap-5">
      <EngineSettings settings={settings} onChange={setSettings} />
      <EngineOverlays
        bestMove={bestMove}
        threatMove={threatMove}
        pieces={pieces}
      />
      <EngineLines
        snapshot={snapshot}
        status={status}
        onToggleStatus={onToggleStatus}
      />
    </div>
  );
};

export default EngineTab;
