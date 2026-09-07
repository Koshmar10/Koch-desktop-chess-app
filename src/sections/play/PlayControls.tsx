import {
  ChessKnight,
  Clock,
  Contrast,
  Flag,
  Flame,
  Loader2,
  Play,
  Share2,
  Swords,
  Zap,
} from "lucide-react";
import { TooltipButton } from "../../components/TooltipButton";
import Dropdown from "../../components/Dropdown";
import {
  useGameContext,
  isGameOngoing,
  type ColorPreference,
  type GameMode,
} from "./GameContext";

const ICON_SIZE = 24;

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

interface ModeOption {
  mode: GameMode;
  label: string;
  time: string;
  icon: React.ReactNode;
}

const MODE_OPTIONS: ModeOption[] = [
  {
    mode: "Bullet",
    label: "bullet",
    time: "1 min",
    icon: <Flame size={ICON_SIZE} className="text-primary" />,
  },
  {
    mode: "Blitz",
    label: "blitz",
    time: "3 min",
    icon: <Zap size={ICON_SIZE} className="text-primary" />,
  },
  {
    mode: "Rapid",
    label: "rapid",
    time: "10 min",
    icon: <Clock size={ICON_SIZE} className="text-primary" />,
  },
  {
    mode: "Classical",
    label: "classical",
    time: "30 min",
    icon: <ChessKnight size={ICON_SIZE} className="text-primary" />,
  },
];

const MODE_TRIGGER_ICON: Record<GameMode, React.ReactNode> = {
  Classical: <ChessKnight size={ICON_SIZE} />,
  Rapid: <Clock size={ICON_SIZE} />,
  Blitz: <Zap size={ICON_SIZE} />,
  Bullet: <Flame size={ICON_SIZE} />,
};

const getStartButtonIcon = (
  isOngoing: boolean,
  isStartingGame: boolean,
): React.ReactNode => {
  if (isOngoing) return <Swords size={ICON_SIZE} className="animate-pulse" />;
  if (isStartingGame) return <Loader2 size={ICON_SIZE} className="animate-spin" />;
  return <Play size={ICON_SIZE} />;
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
    {MODE_OPTIONS.map(({ mode, label, time, icon }, idx) => {
      const isFirst = idx === 0;
      const isLast = idx === MODE_OPTIONS.length - 1;
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
            {icon}
            <span className="capitalize">{label}</span>
          </div>
          <span>{time}</span>
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
  } = useGameContext();

  return (
    <div className="flex flex-row gap-4 items-center self-center mt-4">
      <TooltipButton
        icon={<Flag size={ICON_SIZE} />}
        tooltip="Surrender"
        onClick={() => endGame()}
        disabled={!isGameOngoing(game)}
      />

      <TooltipButton
        icon={getStartButtonIcon(isGameOngoing(game), isStartingGame)}
        tooltip={isGameOngoing(game) ? "Ongoing game" : isStartingGame ? "Starting…" : "Start game"}
        onClick={startGame}
        disabled={isStartingGame || isGameOngoing(game)}
      />

      <TooltipButton
        icon={
          <Contrast
            size={ICON_SIZE}
            className={`transition-transform duration-150 ${COLOR_PREFERENCE_ROTATION[colorPreference]}`}
          />
        }
        tooltip={COLOR_PREFERENCE_TOOLTIP[colorPreference]}
        onClick={cycleColorPreference}
        disabled={isGameOngoing(game)}
      />

      <Dropdown
        side="top"
        align="center"
        trigger={({ open, toggle }) => (
          <TooltipButton
            icon={MODE_TRIGGER_ICON[selectedMode]}
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

      <TooltipButton icon={<Share2 size={ICON_SIZE} />} tooltip="Share game" disabled={isStartingGame || isGameOngoing(game)} />
    </div>
  );
};

export default PlayControls;
