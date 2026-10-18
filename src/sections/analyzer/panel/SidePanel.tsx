import { Cpu, Search, Swords } from "lucide-react";
import { BOARD_SIZE_PX } from "../../../components/chessboard/lib/constants";
import type { PlacedPiece } from "../../../components/chessboard/lib/types";
import type { GameSummary } from "../../../api/bindings/GameSummary";
import type { MoveQuality } from "../../../api/bindings/MoveQuality";
import type { PositionFindings } from "../../../api/bindings/PositionFindings";
import type { SideAccuracy } from "../../../api/bindings/SideAccuracy";
import EngineTab from "./engine/EngineTab";
import GamePanel from "./game/GamePanel";
import PositionPanel from "./position/PositionPanel";
import type { EngineSnapshot, EngineStatus, PanelTab } from "../types";
import { useAnalyzerContext } from "../AnalyzerContext";

const TAB_ICON_SIZE_PX = 14;
// Wider than the board's other neighbours because the Position tab holds
// two colors' worth of king-safety breakdown; at the old 208px the penalty
// labels wrapped every line.
const PANEL_WIDTH_PX = 320;

const TAB_OPTIONS: { tab: PanelTab; label: string; icon: React.ReactNode }[] = [
  { tab: "game", label: "Game", icon: <Swords size={TAB_ICON_SIZE_PX} /> },
  {
    tab: "position",
    label: "Position",
    icon: <Search size={TAB_ICON_SIZE_PX} />,
  },
  { tab: "engine", label: "Engine", icon: <Cpu size={TAB_ICON_SIZE_PX} /> },
];

interface SidePanelProps {
  game: GameSummary | null;
  moves: string[];
  clocks?: (string | null)[];
  qualities?: (MoveQuality | null)[];
  whiteAccuracy: SideAccuracy | null;
  blackAccuracy: SideAccuracy | null;
  viewedPly: number;
  onSelectPly: (ply: number) => void;
  snapshot: EngineSnapshot | null;
  engineStatus: EngineStatus;
  onToggleEngine: () => void;
  onEngineSettingsApplied: () => void;
  // Null while a game is still loading, or if it failed to.
  findings: PositionFindings | null;
  pieces: PlacedPiece[];
  bestMove: string | null;
  threatMove: string | null;
  // The route's mode — only the Game tab's picker cares, for its label.
  isGameMode: boolean;
}

const SidePanel = ({
  game,
  moves,
  clocks,
  qualities,
  whiteAccuracy,
  blackAccuracy,
  viewedPly,
  onSelectPly,
  snapshot,
  engineStatus,
  onToggleEngine,
  onEngineSettingsApplied,
  findings,
  pieces,
  bestMove,
  threatMove,
  isGameMode,
}: SidePanelProps) => {
  // Which tab is open lives in the analyzer's context, so it survives
  // leaving the screen; opening a different game puts it back on Game.
  const {
    game: { selectedTab, setSelectedTab },
    isGameSelectOverlayOpen,
    setIsGameSelectOverlayOpen,
  } = useAnalyzerContext();

  return (
    <div
      className="flex flex-col"
      style={{ width: PANEL_WIDTH_PX, height: BOARD_SIZE_PX }}
    >
      {/* Folder tabs: the active tab has no bottom border and the body has
          no top border, so the two share one continuous outline and read
          as a single object. The filler carries the border line across the
          space to the right of the last tab. Done this way rather than by
          overlapping the tab over the body's border, which only hides it
          if the surface is opaque — and this one isn't. */}
      <div role="tablist" className="flex flex-row">
        {TAB_OPTIONS.map(({ tab: value, label, icon }) => {
          const isActive = selectedTab === value;
          return (
            <button
              key={value}
              type="button"
              role="tab"
              aria-selected={isActive}
              onClick={() => setSelectedTab(value)}
              className={`flex cursor-pointer items-center gap-1.5 rounded-t-md px-3 py-1.5 text-xs transition-colors duration-150 ${
                isActive
                  ? "border-[1px] border-b-0 border-border/80 bg-secondary/40 font-semibold text-foreground"
                  : "border-b-[1px] border-border/80 bg-secondary/15 text-foreground/55 hover:bg-secondary/30 hover:text-foreground/80"
              }`}
            >
              {icon}
              {label}
            </button>
          );
        })}
        <div className="flex-1 border-b-[1px] border-border/80" />
      </div>

      <div className="flex min-h-0 flex-1 flex-col rounded-b-lg border-[1px] border-t-0 border-border/80 bg-secondary/40 p-3 shadow-sm">
        {selectedTab === "game" && (
          <GamePanel
            game={game}
            moves={moves}
            clocks={clocks}
            qualities={qualities}
            whiteAccuracy={whiteAccuracy}
            blackAccuracy={blackAccuracy}
            viewedPly={viewedPly}
            onSelectPly={onSelectPly}
            isGameMode={isGameMode}
            isGameSelectOpen={isGameSelectOverlayOpen}
            onToggleGameSelect={() =>
              setIsGameSelectOverlayOpen((open) => !open)
            }
          />
        )}
        {selectedTab === "position" && (
          <PositionPanel findings={findings} pieces={pieces} />
        )}
        {selectedTab === "engine" && (
          <EngineTab
            snapshot={snapshot}
            status={engineStatus}
            onToggleStatus={onToggleEngine}
            onSettingsApplied={onEngineSettingsApplied}
            bestMove={bestMove}
            threatMove={threatMove}
            pieces={pieces}
          />
        )}
      </div>
    </div>
  );
};

export default SidePanel;
