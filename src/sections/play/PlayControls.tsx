import {
  ChessKnight,
  Clock,
  Flag,
  Flame,
  Play,
  Share2,
  Zap,
} from "lucide-react";
import { TooltipButton } from "../../components/TooltipButton";
import Dropdown from "../../components/Dropdown";
import { useState } from "react";

type GameMode = "Bullet" | "Blitz" | "Rapid" | "Classical";

const ICON_SIZE = 22;

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
  const [selectedGameMode, setSelectedGameMode] = useState<GameMode | null>(
    null,
  );

  return (
    <div className="flex flex-row gap-4 items-center mt-4">
      <TooltipButton icon={<Flag size={ICON_SIZE} />} tooltip="Surrender" />

      <TooltipButton icon={<Play size={ICON_SIZE} />} tooltip="Start game" />

      <Dropdown
        side="top"
        align="center"
        trigger={({ open, toggle }) => (
          <TooltipButton
            icon={
              selectedGameMode ? (
                MODE_TRIGGER_ICON[selectedGameMode]
              ) : (
                <Clock size={ICON_SIZE} />
              )
            }
            tooltip={!open ? "Change game mode" : null}
            onClick={toggle}
          />
        )}
      >
        {(close) => (
          <div className="flex flex-col shadow-md rounded-md min-w-[240px] bg-background">
            {MODE_OPTIONS.map(({ mode, label, time, icon }, idx) => (
              <button
                key={mode}
                className={`px-4 py-2 hover:bg-primary text-left flex justify-between items-center text-foreground ${
                  idx === 0 ? "rounded-t-md" : ""
                } ${idx === MODE_OPTIONS.length - 1 ? "rounded-b-md" : ""} ${
                  selectedGameMode === mode
                    ? "bg-primary font-bold"
                    : "bg-primary/40"
                }`}
                onClick={() => {
                  setSelectedGameMode(mode);
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

      <TooltipButton icon={<Share2 size={ICON_SIZE} />} tooltip="Share game" />
    </div>
  );
};

export default PlayControls;
