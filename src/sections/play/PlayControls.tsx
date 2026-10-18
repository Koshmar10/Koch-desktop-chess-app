import {
  Contrast,
  Flag,
  Loader2,
  Play,
  Share2,
  Swords,
  type LucideIcon,
} from "lucide-react";
import { TooltipButton } from "../../components/TooltipButton";
import Dropdown from "../../components/Dropdown";
import {
  GAME_MODES,
  MODE_ICON,
  modeMinutesLabel,
  type GameMode,
} from "../../lib/gameModes";
import {
  useGameContext,
  isGameOngoing,
  type ColorPreference,
} from "./GameContext";

// For the mode menu's rows. The buttons size their own icons.
const MENU_ICON_SIZE = 24;

// One icon (a circle split into a filled half + an outline half), rotated
// per state instead of swapping to a different icon each time — 180° flips
// which half reads as "filled" for white vs. black, 90° reads as neither.
const COLOR_PREFERENCE_ROTATION: Record<ColorPreference, string> = {
  white: "rotate-0",
  black: "rotate-180",
  random: "rotate-90",
};

const COLOR_PREFERENCE_TOOLTIP: Record<ColorPreference, string> = {
  random: "Random color",
  white: "Play as White",
  black: "Play as Black",
};

// What the start button shows: swords while a game is on, a spinner while
// one starts, play otherwise.
const startButtonIcon = (
  isOngoing: boolean,
  isStartingGame: boolean,
): { icon: LucideIcon; className?: string } => {
  if (isOngoing) return { icon: Swords, className: "animate-pulse" };
  if (isStartingGame) return { icon: Loader2, className: "animate-spin" };
  return { icon: Play };
};

interface ModeDropdownMenuProps {
  close: () => void;
  selectedMode: GameMode;
  setSelectedMode: (mode: GameMode) => void;
}

const ModeDropdownMenu = ({
  close,
  selectedMode,
  setSelectedMode,
}: ModeDropdownMenuProps) => (
  <div className="flex flex-col shadow-md rounded-md min-w-[240px] bg-background">
    {GAME_MODES.map((mode, idx) => {
      const Icon = MODE_ICON[mode];
      const isFirst = idx === 0;
      const isLast = idx === GAME_MODES.length - 1;
      const isSelected = selectedMode === mode;

      const buttonClassName = [
        "px-4 py-2 hover:bg-primary text-left flex justify-between items-center text-foreground",
        isFirst && "rounded-t-md",
        isLast && "rounded-b-md",
        isSelected ? "bg-primary font-bold" : "bg-primary/40",
      ]
        .filter(Boolean)
        .join(" ");

      return (
        <button
          key={mode}
          className={buttonClassName}
          onClick={() => {
            setSelectedMode(mode);
            close();
          }}
        >
          <div className="flex items-center gap-3">
            <Icon size={MENU_ICON_SIZE} className="text-primary" />
            <span>{mode}</span>
          </div>
          <span>{modeMinutesLabel(mode)}</span>
        </button>
      );
    })}
  </div>
);

const PlayControls = () => {
  const {
    game,
    startGame,
    endGame,
    isStartingGame,
    colorPreference,
    cycleColorPreference,
    selectedMode,
    setSelectedMode,
    openSharePgn,
  } = useGameContext();
  const startIcon = startButtonIcon(isGameOngoing(game), isStartingGame);

  return (
    <div className="flex flex-row gap-4 items-center self-center mt-4">
      <TooltipButton
        icon={Flag}
        tooltip="Surrender"
        onClick={() => endGame()}
        disabled={!isGameOngoing(game)}
      />

      <TooltipButton
        icon={startIcon.icon}
        iconClassName={startIcon.className}
        tooltip={
          isGameOngoing(game)
            ? "Ongoing game"
            : isStartingGame
              ? "Starting…"
              : "Start game"
        }
        onClick={startGame}
        disabled={isStartingGame || isGameOngoing(game)}
      />

      <TooltipButton
        icon={Contrast}
        iconClassName={`transition-transform duration-150 ${COLOR_PREFERENCE_ROTATION[colorPreference]}`}
        tooltip={COLOR_PREFERENCE_TOOLTIP[colorPreference]}
        onClick={cycleColorPreference}
        disabled={isGameOngoing(game)}
      />

      <Dropdown
        side="top"
        align="center"
        trigger={({ open, toggle }) => (
          <TooltipButton
            icon={MODE_ICON[selectedMode]}
            tooltip={!open ? "Change game mode" : null}
            onClick={toggle}
            disabled={isGameOngoing(game)}
          />
        )}
      >
        {(close) => (
          <ModeDropdownMenu
            close={close}
            selectedMode={selectedMode}
            setSelectedMode={setSelectedMode}
          />
        )}
      </Dropdown>

      <TooltipButton
        icon={Share2}
        tooltip="Share game"
        onClick={openSharePgn}
        disabled={isStartingGame || !game}
      />
    </div>
  );
};

export default PlayControls;
