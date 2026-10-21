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
import { useNavigate, useParams } from "react-router-dom";
import type { AnalysisStage } from "../../../api/bindings/AnalysisStage";
import Dropdown from "../../../components/Dropdown";
import TooltipButton from "../../../components/TooltipButton";
import {
  START_POSITION_PLY,
  type AnalyzerMode,
  type GameActions,
  type SandboxActions,
} from "../types";
import MoveNavigation from "../../../components/MoveNavigation";
import { useAnalyzerContext } from "../AnalyzerContext";

// For the mode menu's rows. The buttons size their own icons.
const MENU_ICON_SIZE_PX = 18;

// Game shares the side panel's Game-tab icon, so the two read as one idea.
const MODES: AnalyzerMode[] = ["game", "sandbox"];
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

const reanalyzeTooltip = (
  stage: AnalysisStage | undefined,
  canReanalyze: boolean,
): string => {
  if (!canReanalyze) return "Analyse it from History first, to pick your side";
  if (stage?.kind === "Queued") return "Analysis queued";
  if (stage?.kind === "Running") return `Analysing… ${stage.percent}%`;
  return "Re-analyze game";
};

const GameButtons = ({
  analysisStage,
  onReanalyze,
  onTryInSandbox,
  onNextMistake,
}: GameActions) => {
  const isAnalysing =
    analysisStage?.kind === "Queued" || analysisStage?.kind === "Running";
  return (
    <>
      <TooltipButton
        icon={RefreshCw}
        tooltip={reanalyzeTooltip(analysisStage, onReanalyze !== null)}
        onClick={onReanalyze ?? undefined}
        disabled={onReanalyze === null || isAnalysing}
      />
      <TooltipButton
        icon={GitBranch}
        tooltip="Try from here in the sandbox"
        onClick={onTryInSandbox ?? undefined}
        disabled={onTryInSandbox === null}
      />
      <TooltipButton
        icon={TriangleAlert}
        tooltip={onNextMistake ? "Next mistake" : "No mistakes after this move"}
        onClick={onNextMistake ?? undefined}
        disabled={onNextMistake === null}
      />
    </>
  );
};

const SandboxButtons = ({ onReset, onUndo }: SandboxActions) => {
  const { isImportOverlayOpen, setIsImportOverlayOpen } = useAnalyzerContext();
  return (
    <>
      {/* A toggle, like the copy button: the overlay closes by itself
          once something loads, and this closes it without loading. */}
      <TooltipButton
        icon={ClipboardPaste}
        tooltip="Import a FEN or PGN"
        active={isImportOverlayOpen}
        onClick={() => setIsImportOverlayOpen((open) => !open)}
      />
      <TooltipButton
        icon={RotateCcw}
        tooltip="Reset to the starting position"
        onClick={onReset}
      />
      <TooltipButton
        icon={Undo2}
        tooltip={onUndo ? "Undo move" : "No move to undo"}
        onClick={onUndo ?? undefined}
        disabled={onUndo === null}
      />
    </>
  );
};

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
  // Both always given; the mode decides which set of buttons shows.
  gameActions: GameActions;
  sandboxActions: SandboxActions;
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
  gameActions,
  sandboxActions,
}: AnalyzerToolbarProps) => {
  // The mode is the route, so it can't disagree with what's on screen:
  // `/analysis/:gameId` is a game, `/analysis` the sandbox. Switching
  // navigates — to the last game opened, or to the sandbox.
  const { gameId } = useParams();
  const navigate = useNavigate();
  const { loadedGameId, isCopyOverlayOpen, setIsCopyOverlayOpen } =
    useAnalyzerContext();
  const analyzerMode: AnalyzerMode = gameId === undefined ? "sandbox" : "game";
  const ModeIcon = MODE_ICON[analyzerMode];

  const switchTo = (mode: AnalyzerMode) =>
    navigate(
      mode === "game" && loadedGameId !== null
        ? `/analysis/${loadedGameId}`
        : "/analysis",
    );

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
                icon={ModeIcon}
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
                      // No game has been opened yet, so there's none to go
                      // back to — History is where one gets picked.
                      disabled={mode === "game" && loadedGameId === null}
                      className={`flex cursor-pointer items-center gap-2 px-3 py-1 text-left hover:bg-primary/30 disabled:cursor-not-allowed disabled:opacity-40 disabled:hover:bg-transparent ${mode === analyzerMode ? "text-primary" : ""}`}
                      onClick={() => {
                        switchTo(mode);
                        close();
                      }}
                    >
                      <Icon size={MENU_ICON_SIZE_PX} />
                      {mode}
                    </button>
                  );
                })}
              </div>
            )}
          </Dropdown>
          {analyzerMode === "game" ? (
            <GameButtons {...gameActions} />
          ) : (
            <SandboxButtons {...sandboxActions} />
          )}
        </div>

        {/* Above the board rather than in the side panel: scrubbing is the
            thing you do most on this screen, so it belongs on the board's
            own axis where the eye already is. */}
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
          {/* A toggle, like the flip and chat buttons beside it — the
              overlay has no close button of its own, so this is how it
              goes away again. */}
          <TooltipButton
            icon={Copy}
            tooltip="Copy PGN / FEN"
            active={isCopyOverlayOpen}
            onClick={() => setIsCopyOverlayOpen((open) => !open)}
          />
          <TooltipButton
            icon={FlipVertical}
            tooltip="Flip board"
            active={isFlipped}
            onClick={onFlip}
          />
          <TooltipButton
            icon={EyeOff}
            tooltip={
              hasOverlays ? "Clear overlays" : "Nothing drawn on the board"
            }
            onClick={onClearOverlays}
            disabled={!hasOverlays}
          />
          <TooltipButton
            icon={MessageSquareText}
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
