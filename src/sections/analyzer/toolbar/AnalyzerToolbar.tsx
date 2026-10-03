import {
  ClipboardPaste,
  Copy,
  EyeOff,
  FlaskConical,
  FlipVertical,
  GitBranch,
  MessageSquareText,
  RefreshCw,
  RotateCcw,
  Swords,
  TriangleAlert,
  Undo2,
  type LucideIcon,
} from "lucide-react";
import { useState } from "react";
import Dropdown from "../../../components/Dropdown";
import { TooltipButton } from "../../../components/TooltipButton";
import { START_POSITION_PLY } from "../types";
import MoveNavigation from "./MoveNavigation";

const ICON_SIZE = 18;

// Provisional: local state so the two button sets can be compared while
// the design settles. The real mode is the route (`/analysis` is the
// sandbox, `/analysis/:gameId` a game) — switching will navigate.
type AnalyzerMode = "game" | "sandbox";
const MODES: AnalyzerMode[] = ["game", "sandbox"];

// Game shares the side panel's Game-tab icon, so the two read as one idea.
const MODE_ICON: Record<AnalyzerMode, LucideIcon> = {
  game: Swords,
  sandbox: FlaskConical,
};

// Three sections at 40 / 20 / 40 of the toolbar: 2fr + 1fr + 2fr = 5 parts.
// A section whose buttons need more than its share takes what they need,
// and the others split what's left — so the row never overflows the board.
const Group = ({ children }: { children: React.ReactNode }) => (
  <div className="grid w-full grid-cols-[2fr_1fr_2fr] items-center gap-2 rounded-lg border-[1px] border-border/80 bg-secondary/40 px-2">
    {children}
  </div>
);

// Not wired up yet — these are here to see the layout.
const GameButtons = () => (
  <>
    <TooltipButton
      icon={<RefreshCw size={ICON_SIZE} />}
      tooltip="Re-analyze game"
    />
    <TooltipButton
      icon={<GitBranch size={ICON_SIZE} />}
      tooltip="Try from here in the sandbox"
    />
    <TooltipButton
      icon={<TriangleAlert size={ICON_SIZE} />}
      tooltip="Next mistake"
    />
  </>
);

const SandboxButtons = () => (
  <>
    <TooltipButton
      icon={<ClipboardPaste size={ICON_SIZE} />}
      tooltip="Import a FEN or PGN"
    />
    <TooltipButton
      icon={<RotateCcw size={ICON_SIZE} />}
      tooltip="Reset to the starting position"
    />
    <TooltipButton icon={<Undo2 size={ICON_SIZE} />} tooltip="Undo move" />
  </>
);

interface AnalyzerToolbarProps {
  // START_POSITION_PLY (-1) means "before the first move".
  viewedPly: number;
  totalPlies: number;
  // One handler rather than first/prev/next/last: the toolbar turns each
  // button into a target ply itself, and the screen clamps whatever it's
  // handed — so this stays the same function the move list and every
  // other scrubber call.
  onGoToPly: (ply: number) => void;
  isFlipped: boolean;
  onFlip: () => void;
  // Whether anything is drawn — a lit finding or a whole-board overlay.
  // The switches live in the tabs, next to what they draw, so from the
  // Game tab every one that's on is out of sight; this button is the one
  // way to clear them that's visible from anywhere.
  hasOverlays: boolean;
  onClearOverlays: () => void;
  isChatOpen: boolean;
  onToggleChat: () => void;
}

const AnalyzerToolbar = ({
  viewedPly,
  totalPlies,
  onGoToPly,
  isFlipped,
  onFlip,
  hasOverlays,
  onClearOverlays,
  isChatOpen,
  onToggleChat,
}: AnalyzerToolbarProps) => {
  const [analyzerMode, setAnalyzerMode] = useState<AnalyzerMode>("sandbox");
  const ModeIcon = MODE_ICON[analyzerMode];

  return (
    <div className="col-start-2 row-start-1 flex items-center">
      <Group>
        {/* Relative: the mode, and the buttons that only make sense in it. */}
        <div className="flex flex-row items-center gap-2">
          {/* Same size as every other button, so both sides of the
              navigation hold four controls in either mode. */}
          <Dropdown
            trigger={({ open, toggle }) => (
              <TooltipButton
                icon={<ModeIcon size={ICON_SIZE} />}
                tooltip={`Mode: ${analyzerMode}`}
                active={open}
                onClick={toggle}
              />
            )}
          >
            {(close) => (
              <div className="flex flex-col rounded-md border-[1px] border-border bg-card py-1">
                {MODES.map((mode) => {
                  const Icon = MODE_ICON[mode];
                  return (
                    <button
                      key={mode}
                      type="button"
                      className={`flex cursor-pointer items-center gap-2 px-3 py-1 text-left hover:bg-primary/30 ${mode === analyzerMode ? "text-primary" : ""}`}
                      onClick={() => {
                        setAnalyzerMode(mode);
                        close();
                      }}
                    >
                      <Icon size={ICON_SIZE} />
                      {mode}
                    </button>
                  );
                })}
              </div>
            )}
          </Dropdown>
          {analyzerMode === "game" ? <GameButtons /> : <SandboxButtons />}
        </div>

        <div className="flex justify-center">
          <MoveNavigation
            viewedPly={viewedPly}
            totalPlies={totalPlies}
            onFirst={() => onGoToPly(START_POSITION_PLY)}
            onPrev={() => onGoToPly(viewedPly - 1)}
            onNext={() => onGoToPly(viewedPly + 1)}
            onLast={() => onGoToPly(totalPlies - 1)}
          />
        </div>

        {/* Fixed utility: the same in every mode. */}
        <div className="flex flex-row justify-end gap-2">
          <TooltipButton
            icon={<Copy size={ICON_SIZE} />}
            tooltip="Copy PGN / FEN"
          />
          <TooltipButton
            icon={<FlipVertical size={ICON_SIZE} />}
            tooltip="Flip board"
            active={isFlipped}
            onClick={onFlip}
          />
          <TooltipButton
            icon={<EyeOff size={ICON_SIZE} />}
            tooltip={
              hasOverlays ? "Clear overlays" : "Nothing drawn on the board"
            }
            onClick={onClearOverlays}
            disabled={!hasOverlays}
          />
          <TooltipButton
            icon={<MessageSquareText size={ICON_SIZE} />}
            tooltip={isChatOpen ? "Hide assistant" : "Show assistant"}
            active={isChatOpen}
            onClick={onToggleChat}
          />
        </div>
      </Group>
    </div>
  );
};

export default AnalyzerToolbar;
