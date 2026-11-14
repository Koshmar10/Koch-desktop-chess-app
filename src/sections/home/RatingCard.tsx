import { Line, LineChart, ResponsiveContainer, YAxis } from "recharts";
import type { RatingPoint } from "../../api/bindings/RatingPoint";
import { HOME_CARD_CLASS } from "./homeStyles";

const SPARKLINE_HEIGHT_PX = 56;

interface RatingCardProps {
  rating: number | undefined;
  history: RatingPoint[];
}

const RatingCard = ({ rating, history }: RatingCardProps) => {
  const ratings = history.map((point) => point.rating);

  return (
    <div className={`flex w-72 shrink-0 flex-col gap-2 ${HOME_CARD_CLASS}`}>
      <span className="text-xs tracking-wide text-foreground/50 uppercase">
        Rating
      </span>
      <span className="font-mono text-5xl text-foreground">
        {rating ?? "—"}
      </span>
      <div style={{ height: SPARKLINE_HEIGHT_PX }}>
        <ResponsiveContainer width="100%" height="100%">
          <LineChart data={history}>
            <YAxis hide domain={["dataMin", "dataMax"]} />
            <Line
              type="monotone"
              dataKey="rating"
              stroke="var(--primary)"
              strokeWidth={1.5}
              dot={false}
              isAnimationActive={false}
            />
          </LineChart>
        </ResponsiveContainer>
      </div>
      {ratings.length > 0 && (
        <div className="flex justify-between font-mono text-[10px] text-foreground/40">
          <span>low {Math.min(...ratings)}</span>
          <span>peak {Math.max(...ratings)}</span>
        </div>
      )}
    </div>
  );
};

export default RatingCard;
