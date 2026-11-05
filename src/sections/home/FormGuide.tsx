import {
  PolarAngleAxis,
  PolarGrid,
  PolarRadiusAxis,
  Radar,
  RadarChart,
} from "recharts";
import EmptyNote from "../../components/EmptyNote";
import HomeBlock from "./HomeBlock";
import { SAMPLE_FORM } from "./sampleData";

const RADAR_WIDTH_PX = 300;
const RADAR_HEIGHT_PX = 230;
const RADAR_RADIUS_PX = 80;
const SKILL_MAX = 100;

const WEAK_SKILL_TEXT_CLASS = "text-red-400";
const WEAK_SKILL_FILL_CLASS = "fill-red-400";
const SKILL_FILL_CLASS = "fill-foreground/70";

const FormGuide = () => {
  const weakest = SAMPLE_FORM.reduce((low, axis) =>
    axis.recent < low.recent ? axis : low,
  );

  return (
    <HomeBlock
      title="Form guide"
      subtitle="Last 10 games, over your all-time shape."
    >
      <RadarChart
        width={RADAR_WIDTH_PX}
        height={RADAR_HEIGHT_PX}
        outerRadius={RADAR_RADIUS_PX}
        data={SAMPLE_FORM}
        className="self-center"
      >
        <PolarGrid stroke="var(--border)" />
        <PolarRadiusAxis
          domain={[0, SKILL_MAX]}
          tick={false}
          axisLine={false}
        />
        <PolarAngleAxis
          dataKey="skill"
          tick={({ x, y, textAnchor, payload }) => (
            <text
              x={x}
              y={y}
              textAnchor={textAnchor}
              dominantBaseline="central"
              fontSize={11}
              className={
                payload.value === weakest.skill
                  ? WEAK_SKILL_FILL_CLASS
                  : SKILL_FILL_CLASS
              }
            >
              {payload.value}
            </text>
          )}
        />
        <Radar
          dataKey="allTime"
          stroke="var(--muted-foreground)"
          strokeDasharray="4 3"
          fill="none"
          isAnimationActive={false}
        />
        <Radar
          dataKey="recent"
          stroke="var(--primary)"
          fill="var(--primary)"
          fillOpacity={0.35}
          isAnimationActive={false}
        />
      </RadarChart>
      <p className="text-sm text-foreground/70">
        <span className={WEAK_SKILL_TEXT_CLASS}>{weakest.skill}</span> is your
        weak flank lately.
      </p>
      <EmptyNote>Sample data, not from your games yet.</EmptyNote>
    </HomeBlock>
  );
};

export default FormGuide;
