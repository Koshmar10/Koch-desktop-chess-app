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
import { useGameContext, type ColorPreference, type GameMode } from "./GameContext";

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

const MODE_OPTIONS: {
  mode: GameMode;
  label: string;
  time: string;
  icon: React.ReactNode;
}[] = [
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
  const isOngoing = game !== null && game.state.result === "Unfinished";

  return (
    <div className="flex flex-row gap-4 items-center self-center mt-4">
      <TooltipButton
        icon={<Flag size={ICON_SIZE} />}
        tooltip="Surrender"
        onClick={() => endGame()}
        disabled={!isOngoing}
      />

      <TooltipButton
        icon={
          isOngoing ? <Swords size={ICON_SIZE} className="animate-pulse" /> :
            isStartingGame ? (
              <Loader2 size={ICON_SIZE} className="animate-spin" />
            ) : (
              <Play size={ICON_SIZE} />
            )
        }
        tooltip={isOngoing ? "Ongoing game" : isStartingGame ? "Starting…" : "Start game"}
        onClick={startGame}
        disabled={isStartingGame || isOngoing}
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
        disabled={isOngoing}
      />

      <Dropdown
        side="top"
        align="center"
        trigger={({ open, toggle }) => (
          <TooltipButton
            icon={MODE_TRIGGER_ICON[selectedMode]}
            tooltip={!open ? "Change game mode" : null}
            onClick={toggle}
            disabled={isOngoing}
          />
        )}
      >
        {(close) => (
          <div className="flex flex-col shadow-md rounded-md min-w-[240px] bg-background">
            {MODE_OPTIONS.map(({ mode, label, time, icon }, idx) => (
              <button
                key={mode}
                className={`px-4 py-2 hover:bg-primary text-left flex justify-between items-center text-foreground ${idx === 0 ? "rounded-t-md" : ""
                  } ${idx === MODE_OPTIONS.length - 1 ? "rounded-b-md" : ""} ${selectedMode === mode
                    ? "bg-primary font-bold"
                    : "bg-primary/40"
                  }`}
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
            ))}
          </div>
        )}
      </Dropdown>

      <TooltipButton icon={<Share2 size={ICON_SIZE} />} tooltip="Share game" disabled={isStartingGame || isOngoing} />
    </div>
  );
};

export default PlayControls;
