import { useCallback, useEffect, useRef, useState } from "react";
import type { ReactNode } from "react";
import { Target, Clock3, ListChecks } from "lucide-react";
import type { GameAnalysis } from "../../api/bindings/GameAnalysis";
import type { MoveQuality } from "../../api/bindings/MoveQuality";
import type { PieceColor } from "../../api/bindings/PieceColor";
import { CentipawnGraph, TimeGraph } from "./StatGraphs";
import { formatClock } from "./GameContext";

const SLIDE_DURATION_MS = 220;
const SLIDE_DISTANCE_PX = 20;
const AUTO_ADVANCE_MS = 4000;
const SECTION_COUNT = 3;

const MOVE_QUALITY_ORDER: MoveQuality[] = [
  "Brilliant",
  "Great",
  "Excellent",
  "Good",
  "Inaccuracy",
  "Mistake",
  "Blunder",
];

const MOVE_QUALITY_LABEL: Record<MoveQuality, string> = {
  Brilliant: "Brilliant",
  Great: "Great",
  Excellent: "Excellent",
  Good: "Good",
  Inaccuracy: "Inaccuracies",
  Mistake: "Mistakes",
  Blunder: "Blunders",
};

const formatSeconds = (ms: number): string => `${(ms / 1000).toFixed(1)}s`;

interface StatEntry {
  label: string;
  value: string | number;
}

interface StatSection {
  id: string;
  label: string;
  icon: ReactNode;
  graph?: ReactNode;
  stats: StatEntry[];
}

const buildAccuracySection = (analysis: GameAnalysis): StatSection => {
  const counts: Record<MoveQuality, number> = {
    Brilliant: 0,
    Great: 0,
    Excellent: 0,
    Good: 0,
    Inaccuracy: 0,
    Mistake: 0,
    Blunder: 0,
  };
  for (const { quality } of analysis.move_qualities) {
    counts[quality] += 1;
  }

  return {
    id: "accuracy",
    label: "Accuracy",
    icon: <Target size={16} className="text-primary" />,
    stats: [
      { label: "Accuracy", value: `${analysis.accuracy_percent.toFixed(1)}%` },
      {
        label: "Avg. centipawn loss",
        value: analysis.average_centipawn_loss,
      },
      ...MOVE_QUALITY_ORDER.map((quality) => ({
        label: MOVE_QUALITY_LABEL[quality],
        value: counts[quality],
      })),
    ],
  };
};

const buildTimeSection = (analysis: GameAnalysis): StatSection => ({
  id: "time",
  label: "Time",
  icon: <Clock3 size={16} className="text-primary" />,
  graph: <TimeGraph moveTimesMs={analysis.human_move_times_ms} />,
  stats: [
    {
      label: "Avg. time / move",
      value: formatSeconds(analysis.average_move_time_ms),
    },
    {
      label: "Longest think",
      value: analysis.longest_think_ply
        ? `${formatSeconds(analysis.longest_think_ms)} (move ${analysis.longest_think_ply})`
        : formatSeconds(analysis.longest_think_ms),
    },
    { label: "Moves in time trouble", value: analysis.time_trouble_moves },
  ],
});

const buildSummarySection = (
  analysis: GameAnalysis,
  moveHistory: string[],
  humanColor: PieceColor | null,
): StatSection => {
  const worstMove = analysis.move_qualities.reduce<
    (typeof analysis.move_qualities)[number] | null
  >(
    (worst, entry) =>
      !worst || entry.centipawn_loss > worst.centipawn_loss ? entry : worst,
    null,
  );
  const biggestBlunder =
    worstMove && worstMove.quality === "Blunder"
      ? `${moveHistory[worstMove.ply_number - 1] ?? "?"} (move ${worstMove.ply_number})`
      : "None";

  return {
    id: "summary",
    label: "Summary",
    icon: <ListChecks size={16} className="text-primary" />,
    graph: (
      <CentipawnGraph
        history={analysis.centipawn_history}
        humanColor={humanColor}
      />
    ),
    stats: [
      { label: "Total moves", value: moveHistory.length },
      { label: "Game length", value: formatClock(analysis.total_duration_ms) },
      { label: "Biggest blunder", value: biggestBlunder },
    ],
  };
};

const StatRow = ({ label, value }: StatEntry) => (
  <div className="flex items-center justify-between text-md">
    <span className="text-foreground/50">{label}</span>
    <span className="font-semibold text-foreground/90">{value}</span>
  </div>
);

interface PostGameStatsProps {
  analysis: GameAnalysis;
  moveHistory: string[];
  humanColor: PieceColor | null;
}

const PostGameStats = ({
  analysis,
  moveHistory,
  humanColor,
}: PostGameStatsProps) => {
  const statSections = [
    buildAccuracySection(analysis),
    buildTimeSection(analysis),
    buildSummarySection(analysis, moveHistory, humanColor),
  ];

  const [displayedIndex, setDisplayedIndex] = useState(0);
  const [offset, setOffset] = useState(0);
  const [transitionOn, setTransitionOn] = useState(true);
  const isAnimatingRef = useRef(false);
  const pendingIndexRef = useRef<number | null>(null);
  const isHoveredRef = useRef(false);

  const active = statSections[displayedIndex];

  const goTo = useCallback(
    (targetIndex: number) => {
      if (targetIndex === displayedIndex || isAnimatingRef.current) return;
      isAnimatingRef.current = true;
      pendingIndexRef.current = targetIndex;
      setTransitionOn(true);
      setOffset(-SLIDE_DISTANCE_PX);
    },
    [displayedIndex],
  );

  const handleTransitionEnd = (e: React.TransitionEvent) => {
    if (e.propertyName !== "transform") return;

    if (offset !== 0 && pendingIndexRef.current !== null) {
      setDisplayedIndex(pendingIndexRef.current);
      pendingIndexRef.current = null;
      setTransitionOn(false);
      setOffset(SLIDE_DISTANCE_PX);
      requestAnimationFrame(() => {
        requestAnimationFrame(() => {
          setTransitionOn(true);
          setOffset(0);
        });
      });
    } else if (offset === 0) {
      isAnimatingRef.current = false;
    }
  };

  useEffect(() => {
    const interval = setInterval(() => {
      if (isHoveredRef.current) return;
      goTo((displayedIndex + 1) % SECTION_COUNT);
    }, AUTO_ADVANCE_MS);
    return () => clearInterval(interval);
  }, [displayedIndex, goTo]);

  return (
    <div className="flex flex-col items-center gap-3 w-72">
      <div className="flex items-center gap-2">
        {statSections.map((section, idx) => (
          <button
            key={section.id}
            type="button"
            aria-label={`Show ${section.label} stats`}
            onClick={() => goTo(idx)}
            className={`rounded-full transition-all duration-150 ${idx === displayedIndex
              ? "w-2.5 h-2.5 bg-primary"
              : "w-2 h-2 bg-foreground/20 hover:bg-foreground/40"
              }`}
          />
        ))}
      </div>

      <div
        onMouseEnter={() => (isHoveredRef.current = true)}
        onMouseLeave={() => (isHoveredRef.current = false)}
        onTransitionEnd={handleTransitionEnd}
        className="flex flex-col gap-3 p-4 rounded-lg bg-secondary/60 shadow-sm border-[1px] border-border/80 w-full h-86"
        style={{
          transform: `translateX(${offset}px)`,
          opacity: offset === 0 ? 1 : 0,
          transition: transitionOn
            ? `transform ${SLIDE_DURATION_MS}ms ease, opacity ${SLIDE_DURATION_MS}ms ease`
            : "none",
        }}
      >
        <h3 className="flex items-center gap-2 text-sm font-semibold text-foreground/60 uppercase tracking-wide">
          {active.icon}
          {active.label}
        </h3>

        <div className="flex flex-col gap-2">
          {active.stats.map((stat: StatEntry) => (
            <StatRow key={stat.label} {...stat} />
          ))}
        </div>
        {active.graph}
      </div>
    </div>
  );
};

export default PostGameStats;
