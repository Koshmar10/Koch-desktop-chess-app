import {
  Download,
  EyeOff,
  FileText,
  FlipVertical,
  MessageSquareText,
} from "lucide-react";
import { TooltipButton } from "../../../components/TooltipButton";
import { START_POSITION_PLY } from "../types";
import MoveNavigation from "./MoveNavigation";

const ICON_SIZE = 18;

const Group = ({ children }: { children: React.ReactNode }) => (
  <div className="flex w-full flex-row items-center justify-center gap-4 rounded-lg border-[1px] border-border/80 bg-secondary/40">
    {children}
  </div>
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
}: AnalyzerToolbarProps) => (
  <div className="col-start-2 row-start-1 flex items-center">
    <Group>
      {/* Board orientation and what's drawn on it. */}
      <TooltipButton
        icon={<FlipVertical size={ICON_SIZE} />}
        tooltip="Flip board"
        active={isFlipped}
        onClick={onFlip}
      />
      <TooltipButton
        icon={<EyeOff size={ICON_SIZE} />}
        tooltip={hasOverlays ? "Clear overlays" : "Nothing drawn on the board"}
        onClick={onClearOverlays}
        disabled={!hasOverlays}
      />

      <MoveNavigation
        viewedPly={viewedPly}
        totalPlies={totalPlies}
        onFirst={() => onGoToPly(START_POSITION_PLY)}
        onPrev={() => onGoToPly(viewedPly - 1)}
        onNext={() => onGoToPly(viewedPly + 1)}
        onLast={() => onGoToPly(totalPlies - 1)}
      />

      {/* Both of these need a loaded game to act on, which arrives with
          KOCH-12 — disabled rather than hidden so the toolbar doesn't
          change shape once they start working. */}
      <TooltipButton
        icon={<Download size={ICON_SIZE} />}
        tooltip="Export game"
        disabled
      />
      <TooltipButton
        icon={<FileText size={ICON_SIZE} />}
        tooltip="Game report"
        disabled
      />
      <TooltipButton
        icon={<MessageSquareText size={ICON_SIZE} />}
        tooltip={isChatOpen ? "Hide assistant" : "Show assistant"}
        active={isChatOpen}
        onClick={onToggleChat}
      />
    </Group>
  </div>
);

export default AnalyzerToolbar;
