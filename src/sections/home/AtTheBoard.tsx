import type { PlayerStats } from "../../api/bindings/PlayerStats";
import { formatDuration } from "../../lib/time";
import HomeBlock from "./HomeBlock";

interface FigureProps {
  label: string;
  value: string;
}

const Figure = ({ label, value }: FigureProps) => (
  <div className="flex flex-col">
    <span className="text-[10px] text-foreground/50">{label}</span>
    <span className="font-mono text-sm text-foreground">{value}</span>
  </div>
);

interface AtTheBoardProps {
  stats: PlayerStats | null;
}

const AtTheBoard = ({ stats }: AtTheBoardProps) => (
  <HomeBlock title="At the board" subtitle="The numbers behind your play.">
    <div className="flex flex-col gap-1">
      <span className="text-[10px] text-foreground/50">Time played</span>
      <span className="font-mono text-3xl text-foreground">
        {stats ? formatDuration(stats.total_seconds) : "—"}
      </span>
    </div>
    <div className="grid grid-cols-3 gap-2 border-t-[1px] border-border/60 pt-3">
      <Figure
        label="Accuracy"
        value={stats ? `${stats.avg_accuracy.toFixed(1)}%` : "—"}
      />
      <Figure
        label="Best streak"
        value={stats ? `${stats.best_streak}` : "—"}
      />
      <Figure
        label="Win rate"
        value={stats ? `${Math.round(stats.win_rate)}%` : "—"}
      />
    </div>
  </HomeBlock>
);

export default AtTheBoard;
