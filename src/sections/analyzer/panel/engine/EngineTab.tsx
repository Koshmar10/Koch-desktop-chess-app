import type { PlacedPiece } from "../../../../components/chessboard/lib/types";
import EngineLines from "./EngineLines";
import EngineOverlays from "./EngineOverlays";
import EngineSettings from "./EngineSettings";
import { useAnalyzerEngineSettings } from "./useAnalyzerEngineSettings";
import type {
  ActiveOverlays,
  EngineSnapshot,
  EngineStatus,
  Overlay,
} from "../../types";

interface EngineTabProps {
  snapshot: EngineSnapshot | null;
  status: EngineStatus;
  onToggleStatus: () => void;
  bestMove: string | null;
  threatMove: string | null;
  pieces: PlacedPiece[];
  overlays: ActiveOverlays;
  onToggleOverlay: (overlay: Overlay) => void;
}

// Settings and the board switches first at their natural height, then the
// lines taking whatever's left and scrolling on their own — so a growing
// line list never pushes the controls around, and the controls sit at a
// stable position you can reach for without looking.
const EngineTab = ({
  snapshot,
  status,
  onToggleStatus,
  bestMove,
  threatMove,
  pieces,
  overlays,
  onToggleOverlay,
}: EngineTabProps) => {
  const [settings, setSettings] = useAnalyzerEngineSettings();

  return (
    <div className="flex min-h-0 flex-1 flex-col gap-5">
      <EngineSettings settings={settings} onChange={setSettings} />
      <EngineOverlays
        bestMove={bestMove}
        threatMove={threatMove}
        pieces={pieces}
        overlays={overlays}
        onToggleOverlay={onToggleOverlay}
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
