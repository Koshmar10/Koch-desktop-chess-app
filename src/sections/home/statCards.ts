import type { LucideIcon } from "lucide-react";
import { TrendingUp, Trophy, Target, Medal, Zap, Clock } from "lucide-react";
import type { PlayerStats } from "../../api/bindings/PlayerStats";
import { formatDuration } from "../../lib/time";

// Layout + presentation for each Home stat card. The value comes from live
// `PlayerStats` via `format`; the icon / label / grid span are fixed here.
export interface StatCard {
  icon: LucideIcon;
  label: string;
  className: string;
  delta?: string;
  format: (stats: PlayerStats) => string | number;
}

export const STAT_CARDS: StatCard[] = [
  {
    icon: TrendingUp,
    label: "Current Rating",
    className: "row-span-2 col-span-1",
    format: (s) => s.current_rating,
  },
  {
    icon: Medal,
    label: "Avg. Accuracy",
    className: "row-span-2 col-span-1",
    format: (s) => `${s.avg_accuracy.toFixed(1)}%`,
  },
  {
    icon: Trophy,
    label: "Win Rate",
    className: "row-span-1 col-span-2",
    format: (s) => `${Math.round(s.win_rate)}%`,
  },
  {
    icon: Clock,
    label: "Total Time",
    className: "row-span-2 col-span-1",
    format: (s) => formatDuration(s.total_seconds),
  },
  {
    icon: Zap,
    label: "Best Streak",
    delta: "wins",
    className: "row-span-2 col-span-1",
    format: (s) => s.best_streak,
  },
  {
    icon: Target,
    label: "Games Played",
    className: "row-span-1 col-span-2",
    format: (s) => s.games_played,
  },
];
