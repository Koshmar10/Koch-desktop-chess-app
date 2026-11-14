import type { FormGuide as FormGuideData } from "../../api/bindings/FormGuide";
import type { FormSkill } from "../../api/bindings/FormSkill";
import {
  PolarAngleAxis,
  PolarGrid,
  PolarRadiusAxis,
  Radar,
  RadarChart,
} from "recharts";
import EmptyNote from "../../components/EmptyNote";
import HomeBlock from "./HomeBlock";

const RADAR_WIDTH_PX = 300;
const RADAR_HEIGHT_PX = 230;
const RADAR_RADIUS_PX = 80;
const SKILL_MAX = 100;

const WEAK_SKILL_TEXT_CLASS = "text-red-400";
const WEAK_SKILL_FILL_CLASS = "fill-red-400";
const SKILL_FILL_CLASS = "fill-foreground/70";

const SKILL_LABEL: Record<FormSkill, string> = {
  opening: "Opening",
  middlegame: "Middlegame",
  endgame: "Endgame",
  tactics: "Tactics",
  defence: "Defence",
  converting: "Converting",
};

interface FormGuideProps {
  // Null until it loads.
  form: FormGuideData | null;
}

const FormGuide = ({ form }: FormGuideProps) => {
  const scored = form?.axes.filter((axis) => axis.recent !== null) ?? [];
  // A skill those games gave nothing to score sits at the centre, like a
  // zero, but is never called the weak one.
  const data =
    form?.axes.map((axis) => ({
      skill: SKILL_LABEL[axis.skill],
      recent: axis.recent ?? 0,
      allTime: axis.all_time ?? 0,
    })) ?? [];
  const weakest = scored.reduce<(typeof scored)[number] | null>(
    (low, axis) =>
      low === null || (axis.recent ?? 0) < (low.recent ?? 0) ? axis : low,
    null,
  );
  const games = form?.recent_games ?? 0;

  return (
    <HomeBlock
      title="Form guide"
      subtitle={
        games > 0
          ? `Last ${games === 1 ? "game" : `${games} games`}, over your all-time shape.`
          : "Your last ten games, over your all-time shape."
      }
    >
      {scored.length === 0 ? (
        <EmptyNote>Analyse a game to see your form.</EmptyNote>
      ) : (
        <>
          <RadarChart
            width={RADAR_WIDTH_PX}
            height={RADAR_HEIGHT_PX}
            outerRadius={RADAR_RADIUS_PX}
            data={data}
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
                    weakest && payload.value === SKILL_LABEL[weakest.skill]
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
          {weakest && (
            <p className="text-sm text-foreground/70">
              <span className={WEAK_SKILL_TEXT_CLASS}>
                {SKILL_LABEL[weakest.skill]}
              </span>{" "}
              is your weak flank lately.
            </p>
          )}
        </>
      )}
    </HomeBlock>
  );
};

export default FormGuide;
