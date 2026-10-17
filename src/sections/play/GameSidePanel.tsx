import { Clock3, Gamepad2, TrendingUp } from "lucide-react";
import MoveTimeline from "../../components/MoveTimeline";
import PanelSection from "../../components/PanelSection";
import { useGameContext } from "./GameContext";

// Mock, deliberately — there's no player-stats backend at all yet.
const MOCK_SESSION_STATS = {
  eloRampup: "+42",
  timePlayed: "4h 32m",
  gamesPlayed: 27,
};

interface StatRowProps {
  icon: React.ReactNode;
  label: string;
  value: string | number;
}

const StatRow = ({ icon, label, value }: StatRowProps) => (
  <div className="flex items-center gap-2 text-md">
    {icon}
    <span className="text-foreground/50">{label}</span>
    <span className="ml-auto font-semibold text-foreground/90">{value}</span>
  </div>
);

const GameSidePanel = () => {
  const { game } = useGameContext();
  const moves = game?.state.move_history ?? [];
  const openingName = game?.state.opening_name ?? "Not yet known";

  return (
    <div className="flex flex-col gap-4 w-72 h-210 py-1">
      <div className="p-3 rounded-lg bg-secondary/40 shadow-sm border-[1px] border-border/80">
        <PanelSection title="Session">
          <StatRow
            icon={<TrendingUp size={16} className="text-primary" />}
            label="Elo rampup"
            value={MOCK_SESSION_STATS.eloRampup}
          />
          <StatRow
            icon={<Clock3 size={16} className="text-primary" />}
            label="Time played"
            value={MOCK_SESSION_STATS.timePlayed}
          />
          <StatRow
            icon={<Gamepad2 size={16} className="text-primary" />}
            label="Games played"
            value={MOCK_SESSION_STATS.gamesPlayed}
          />
        </PanelSection>
      </div>

      <div className="flex min-h-0 flex-col gap-4 p-3 rounded-lg bg-secondary/40 shadow-sm border-[1px] border-border/80 h-[72%]">
        <PanelSection title="Opening" divided>
          <span className="text-md text-foreground/80">{openingName}</span>
        </PanelSection>
        <PanelSection title="Moves" fill>
          {/* The board always shows the latest move while you play, so
              that's the one marked — and the list follows it down. */}
          <MoveTimeline moves={moves} viewedPly={moves.length - 1} />
        </PanelSection>
      </div>
    </div>
  );
};

export default GameSidePanel;
