import { useEffect, useState } from "react";
import { ArrowRight } from "lucide-react";
import { Link } from "react-router-dom";
import { getPuzzleStats } from "../../api/puzzle";
import type { PuzzleStats } from "../../api/bindings/PuzzleStats";
import EmptyNote from "../../components/EmptyNote";
import { formatPuzzleTheme } from "../../lib/puzzleThemes";
import HomeBlock from "./HomeBlock";

const NUMBER_CLASS = "font-mono text-2xl text-foreground";

const Figure = ({ label, value }: { label: string; value: string }) => (
  <div className="flex min-w-0 flex-col gap-1">
    <span className="text-[10px] text-foreground/50">{label}</span>
    <span className={`truncate ${NUMBER_CLASS}`}>{value}</span>
  </div>
);

const PuzzleRecord = () => {
  const [stats, setStats] = useState<PuzzleStats | null>(null);

  useEffect(() => {
    getPuzzleStats().then(setStats).catch(console.error);
  }, []);

  return (
    <HomeBlock
      title="Puzzle record"
      subtitle="Every puzzle you've solved or given up on."
    >
      {stats === null || stats.attempted === 0 ? (
        <EmptyNote>Solve a puzzle and your record starts here.</EmptyNote>
      ) : (
        <div className="grid grid-cols-6 gap-6">
          <Figure label="Solved" value={`${stats.solved}/${stats.attempted}`} />
          <Figure
            label="Clean solves"
            value={`${Math.round(stats.clean_rate)}%`}
          />
          <Figure label="Best streak" value={`${stats.best_streak}`} />
          <Figure label="Points" value={`${stats.total_points}`} />
          <Figure
            label="Avg. rating solved"
            value={stats.avg_solved_rating?.toString() ?? "—"}
          />
          <div className="flex min-w-0 flex-col gap-1">
            <span className="text-[10px] text-foreground/50">
              Weakest theme
            </span>
            <span className="truncate text-lg text-red-400">
              {stats.weakest_theme
                ? formatPuzzleTheme(stats.weakest_theme)
                : "—"}
            </span>
          </div>
        </div>
      )}
      <Link
        to="/puzzle"
        className="flex items-center gap-1 self-start text-xs text-primary hover:underline"
      >
        Solve puzzles <ArrowRight size={12} />
      </Link>
    </HomeBlock>
  );
};

export default PuzzleRecord;
