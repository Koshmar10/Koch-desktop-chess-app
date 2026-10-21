import { Flame } from "lucide-react";
import { MAX_STREAK_MULTIPLIER, streakMultiplier } from "./scoring";

const ICON_SIZE_PX = 16;

interface StreakGaugeProps {
  streak: number;
}

const StreakGauge = ({ streak }: StreakGaugeProps) => {
  const multiplier = streakMultiplier(streak);
  const fill = (multiplier - 1) / (MAX_STREAK_MULTIPLIER - 1);

  return (
    <div className="flex flex-col gap-1.5">
      <div className="flex items-center gap-2 text-md">
        <Flame
          size={ICON_SIZE_PX}
          className={streak > 0 ? "text-orange-500" : "text-foreground/30"}
        />
        <span className="text-foreground/50">Streak</span>
        <span className="ml-auto font-semibold text-foreground/90">
          {streak}
        </span>
        <span className="text-xs tabular-nums text-foreground/50">
          ×{multiplier.toFixed(1)}
        </span>
      </div>
      <div className="h-1.5 w-full overflow-hidden rounded-full bg-foreground/10">
        <div
          className="h-full rounded-full bg-orange-500 transition-[width] duration-300"
          style={{ width: `${fill * 100}%` }}
        />
      </div>
    </div>
  );
};

export default StreakGauge;
