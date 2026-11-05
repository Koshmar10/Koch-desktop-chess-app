import { Cell, Pie, PieChart } from "recharts";
import type { GameSummary } from "../../api/bindings/GameSummary";
import type { PieceColor } from "../../api/bindings/PieceColor";
import EmptyNote from "../../components/EmptyNote";
import {
  OTHER_OPENINGS_LABEL,
  openingShares,
  type OpeningShare,
} from "../../lib/homeStats";
import HomeBlock from "./HomeBlock";

const PIE_SIZE_PX = 150;

// Brightest for the opening you play most; "Other" is always the dimmest.
const SLICE_FILLS = [
  "var(--primary)",
  "color-mix(in srgb, var(--primary) 70%, var(--background))",
  "color-mix(in srgb, var(--primary) 45%, var(--background))",
];
const OTHER_FILL = "var(--muted)";

const PIE_COLORS: { color: PieceColor; label: string }[] = [
  { color: "white", label: "As White" },
  { color: "black", label: "As Black" },
];

const sliceFill = (share: OpeningShare, index: number) =>
  share.name === OTHER_OPENINGS_LABEL ? OTHER_FILL : SLICE_FILLS[index];

interface OpeningPieProps {
  label: string;
  shares: OpeningShare[];
}

const OpeningPie = ({ label, shares }: OpeningPieProps) => (
  <div className="flex min-w-0 flex-1 flex-col items-center gap-3">
    <span className="self-start text-xs font-semibold text-foreground">
      {label}
    </span>
    {shares.length === 0 ? (
      <EmptyNote>No games</EmptyNote>
    ) : (
      <>
        <PieChart width={PIE_SIZE_PX} height={PIE_SIZE_PX}>
          <Pie
            data={shares}
            dataKey="count"
            stroke="var(--background)"
            isAnimationActive={false}
          >
            {shares.map((share, index) => (
              <Cell key={share.name} fill={sliceFill(share, index)} />
            ))}
          </Pie>
        </PieChart>
        <ul className="flex w-full flex-col gap-1">
          {shares.map((share, index) => (
            <li key={share.name} className="flex items-center gap-2 text-xs">
              <span
                className="size-2 shrink-0 rounded-[1px]"
                style={{ background: sliceFill(share, index) }}
              />
              <span className="min-w-0 flex-1 truncate text-foreground">
                {share.name}
              </span>
              <span className="font-mono text-foreground/60">
                {share.percent}%
              </span>
            </li>
          ))}
        </ul>
      </>
    )}
  </div>
);

interface OpeningLinesProps {
  games: GameSummary[];
}

const OpeningLines = ({ games }: OpeningLinesProps) => (
  <HomeBlock
    title="Opening lines"
    subtitle="Share of your games that reached each opening."
  >
    <div className="flex gap-6">
      {PIE_COLORS.map(({ color, label }) => (
        <OpeningPie
          key={color}
          label={label}
          shares={openingShares(games, color)}
        />
      ))}
    </div>
  </HomeBlock>
);

export default OpeningLines;
