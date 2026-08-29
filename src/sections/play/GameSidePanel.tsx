import { Clock3, Gamepad2, TrendingUp } from "lucide-react";
import { useGameContext } from "./GameContext";

// Mock, deliberately — there's no player-stats backend at all yet.
const MOCK_SESSION_STATS = {
  eloRampup: "+42",
  timePlayed: "4h 32m",
  gamesPlayed: 27,
};

const movePairs = (moves: string[]): [string, string | undefined][] => {
  const pairs: [string, string | undefined][] = [];
  for (let i = 0; i < moves.length; i += 2) {
    pairs.push([moves[i], moves[i + 1]]);
  }
  return pairs;
};

const MoveList = ({ moves }: { moves: string[] }) => {
  const pairs = movePairs(moves);

  if (pairs.length === 0) {
    return <p className="text-sm text-foreground/40 italic">No moves yet</p>;
  }

  return (
    <div className="flex-1 min-h-0 flex flex-col gap-1 overflow-y-auto pr-1">
      {pairs.map(([white, black], idx) => (
        <div
          key={idx}
          className="flex gap-2 text-md px-2 py-1 rounded hover:bg-primary/10"
        >
          <span className="w-6 text-foreground/40">{idx + 1}.</span>
          <span className="flex-1 text-foreground/90">{white}</span>
          <span className="flex-1 text-foreground/90">{black ?? ""}</span>
        </div>
      ))}
    </div>
  );
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

  return (
    <div className="flex flex-col gap-4 w-72 py-1">

      <div className="flex flex-col gap-3 p-3 rounded-lg bg-secondary/40 shadow-sm border-[1px] border-border/80">
        <h3 className="text-sm font-semibold text-foreground/60 uppercase tracking-wide">
          Session
        </h3>
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
      </div>

      <div className="flex flex-col gap-2 p-3 rounded-lg bg-secondary/40 shadow-sm border-[1px] border-border/80 h-[72%]">
        <h3 className="text-sm font-semibold text-foreground/60 uppercase tracking-wide">
          Opening
        </h3>
        <h2 className="text-md font-normal text-foreground/80 tracking-wide">
          King's Indian defense
        </h2>
        <h3 className="text-sm font-semibold text-foreground/60 uppercase tracking-wide">
          Moves
        </h3>
        <MoveList moves={moves} />
      </div>
    </div>
  );
};

export default GameSidePanel;
