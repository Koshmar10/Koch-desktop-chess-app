import {
  TrendingUp,
  Trophy,
  Target,
  Medal,
  Zap,
  Clock,
  LucideIcon,
} from "lucide-react";

export interface StatCard {
  icon: LucideIcon;
  value: string | number;
  label: string;
  delta?: string;
  className: string;
}

export const STAT_CARDS: StatCard[] = [
  {
    icon: TrendingUp,
    value: 1450,
    label: "Current Rating",
    delta: "+32 this month",
    className: "row-span-2 col-span-1",
  },
  {
    icon: Medal,
    value: "87.3%",
    label: "Avg. Accuracy",
    className: "row-span-2 col-span-1",
  },
  {
    icon: Trophy,
    value: "62%",
    label: "Win Rate",
    delta: "+5%",
    className: "row-span-1 col-span-2",
  },
  {
    icon: Clock,
    value: "48h",
    label: "Total Time",
    className: "row-span-2 col-span-1",
  },
  {
    icon: Zap,
    value: 12,
    label: "Best Streak",
    delta: "wins",
    className: "row-span-2 col-span-1",
  },
  {
    icon: Target,
    value: 214,
    label: "Games Played",
    className: "row-span-1 col-span-2",
  },
];
