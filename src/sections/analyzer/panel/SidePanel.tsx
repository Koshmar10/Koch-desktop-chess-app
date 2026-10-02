import { useState } from "react";
import { Cpu, Search, Swords } from "lucide-react";
import { BOARD_PIXEL_SIZE } from "../../../components/chessboard/lib/constants";
import type { PlacedPiece } from "../../../components/chessboard/lib/types";
import type { GameSummary } from "../../../api/bindings/GameSummary";
import type { MoveQuality } from "../../../api/bindings/MoveQuality";
import type { PositionFindings } from "../../../api/bindings/PositionFindings";
import type { SideAccuracy } from "../../../api/bindings/SideAccuracy";
import EngineTab from "./engine/EngineTab";
import GamePanel from "./game/GamePanel";
import PositionPanel from "./position/PositionPanel";
import type { EngineSnapshot, EngineStatus, MaskSelection } from "../types";

const TAB_ICON_SIZE = 14;
// Wider than the board's other neighbours because the Position tab holds
// two colors' worth of king-safety breakdown; at the old 208px the penalty
// labels wrapped every line.
const PANEL_WIDTH_PX = 320;

/**
 * Ordered widest scope first: the game as a whole, then the position in
 * front of you, then what the engine makes of it.
 *
 * Engine and Position are also split by *what changes them* — Engine
 * repaints several times a second while a search runs, Position only when
 * the viewed ply changes. Interleaving those in one scroll means half the
 * panel is always flickering under the half you're reading.
 */
export type PanelTab = "game" | "position" | "engine";

const TAB_OPTIONS: { tab: PanelTab; label: string; icon: React.ReactNode }[] = [
  { tab: "game", label: "Game", icon: <Swords size={TAB_ICON_SIZE} /> },
  { tab: "position", label: "Position", icon: <Search size={TAB_ICON_SIZE} /> },
  { tab: "engine", label: "Engine", icon: <Cpu size={TAB_ICON_SIZE} /> },
];

interface SidePanelProps {
  // Which tab to open on. Left out, it's Game when there's a game and
  // Engine when there isn't — but a game that's still loading has no
  // summary yet, so the caller says so explicitly.
  initialTab?: PanelTab;
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
  // Null while a game is still loading, or if it failed to.
  findings: PositionFindings | null;
  pieces: PlacedPiece[];
  // What's on the board, shared by the Position and Engine tabs' rows.
  maskSelection: MaskSelection;
  bestMove: string | null;
  threatMove: string | null;
}

const SidePanel = ({
  initialTab,
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
  findings,
  pieces,
  maskSelection,
  bestMove,
  threatMove,
}: SidePanelProps) => {
  // Which tab is open is panel-local: nothing outside this column cares,
  // and KOCH-11's reducer only needs it if something ever has to open the
  // panel *to* a tab (a timeline blunder marker jumping to Position, say).
  //
  // Opens on Game when there's a game to show and Engine otherwise, so
  // sandbox mode never lands you on the one tab that has nothing in it.
  const [tab, setTab] = useState<PanelTab>(
    initialTab ?? (game === null ? "engine" : "game"),
  );

  return (
    <div
      className="flex flex-col"
      style={{ width: PANEL_WIDTH_PX, height: BOARD_PIXEL_SIZE }}
    >
      {/* Folder tabs: the active tab has no bottom border and the body has
          no top border, so the two share one continuous outline and read
          as a single object. The filler carries the border line across the
          space to the right of the last tab. Done this way rather than by
          overlapping the tab over the body's border, which only hides it
          if the surface is opaque — and this one isn't. */}
      <div role="tablist" className="flex flex-row">
        {TAB_OPTIONS.map(({ tab: value, label, icon }) => {
          const isActive = tab === value;
          return (
            <button
              key={value}
              type="button"
              role="tab"
              aria-selected={isActive}
              onClick={() => setTab(value)}
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
        {tab === "game" && (
          <GamePanel
            game={game}
            moves={moves}
            clocks={clocks}
            qualities={qualities}
            whiteAccuracy={whiteAccuracy}
            blackAccuracy={blackAccuracy}
            viewedPly={viewedPly}
            onSelectPly={onSelectPly}
          />
        )}
        {tab === "position" && (
          <PositionPanel
            findings={findings}
            pieces={pieces}
            maskSelection={maskSelection}
          />
        )}
        {tab === "engine" && (
          <EngineTab
            snapshot={snapshot}
            status={engineStatus}
            onToggleStatus={onToggleEngine}
            bestMove={bestMove}
            threatMove={threatMove}
            pieces={pieces}
            maskSelection={maskSelection}
          />
        )}
      </div>
    </div>
  );
};

export default SidePanel;
