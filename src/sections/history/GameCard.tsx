import { useState } from "react";
import {
  formatDatePlayed,
  getScoreCardStyle,
  resultToken,
} from "../../lib/gameSummary";
import {
  MODE_ICON,
  modeForTimeControl,
  type GameMode,
} from "../../lib/gameModes";
import {
  ChessKing,
  ChartSpline,
  MoreVertical,
  Eye,
  Download,
  Trash2,
  RotateCcw,
  Check,
  X,
  Hourglass,
  TriangleAlert,
  Import,
} from "lucide-react";
import type { GameSummary } from "../../api/bindings/GameSummary";
import type { PieceColor } from "../../api/bindings/PieceColor";
import type { AnalysisStage } from "../../api/bindings/AnalysisStage";
import { useAnalysisStage } from "../../app/AppContext";
import Dropdown from "../../components/Dropdown";

interface GameCardProps {
  gameSummary: GameSummary;
  onOpen?: (game: GameSummary) => void;
  onAnalyze?: (game: GameSummary) => void;
  onExportPgn?: (game: GameSummary) => void;
  onDelete?: (game: GameSummary) => void;
}

const MODE_ICON_SIZE_PX = 18;

// The card's own touch: each mode's icon in its own colour, so a page of
// cards sorts itself by speed at a glance. The icons are shared.
const MODE_COLOR_CLASS: Record<GameMode, string> = {
  Bullet: "text-red-500",
  Blitz: "text-amber-500",
  Rapid: "text-sky-600",
  Classical: "text-violet-400",
};

const ModeIcon = ({ mode }: { mode: GameMode }) => {
  const Icon = MODE_ICON[mode];
  return <Icon size={MODE_ICON_SIZE_PX} className={MODE_COLOR_CLASS[mode]} />;
};

const PLAYER_DOT_CLASS: Record<PieceColor, string> = {
  white: "bg-neutral-100 border border-white",
  black: "bg-neutral-900 border border-white",
};
const CardSeparator = () => <hr className="border-border my-[1px]" />;

const ANALYSIS_ICON_SIZE_PX = 18;

const AnalysisRing = ({ percent }: { percent: number }) => {
  const radius = 8;
  const circumference = 2 * Math.PI * radius;
  return (
    <span
      title={`Analysing… ${percent}%`}
      className="relative inline-flex animate-pulse items-center justify-center"
      style={{ width: ANALYSIS_ICON_SIZE_PX, height: ANALYSIS_ICON_SIZE_PX }}
    >
      <svg
        width={ANALYSIS_ICON_SIZE_PX}
        height={ANALYSIS_ICON_SIZE_PX}
        className="-rotate-90"
      >
        <circle
          cx={ANALYSIS_ICON_SIZE_PX / 2}
          cy={ANALYSIS_ICON_SIZE_PX / 2}
          r={radius}
          fill="none"
          strokeWidth={2}
          className="stroke-border"
        />
        <circle
          cx={ANALYSIS_ICON_SIZE_PX / 2}
          cy={ANALYSIS_ICON_SIZE_PX / 2}
          r={radius}
          fill="none"
          strokeWidth={2}
          strokeLinecap="round"
          className="stroke-primary transition-[stroke-dashoffset] duration-300"
          strokeDasharray={circumference}
          strokeDashoffset={circumference * (1 - percent / 100)}
        />
      </svg>
      <span className="absolute text-[7px] font-semibold text-foreground/70">
        {percent}
      </span>
    </span>
  );
};

// A `Done` event counts as analysed straight away, before the list refetches.
const AnalysisIndicator = ({
  hasAnalysis,
  stage,
}: {
  hasAnalysis: boolean;
  stage: AnalysisStage | undefined;
}) => {
  if (stage?.kind === "Running")
    return <AnalysisRing percent={stage.percent} />;
  if (stage?.kind === "Queued") {
    return (
      <span title="Analysis queued">
        <Hourglass
          size={ANALYSIS_ICON_SIZE_PX}
          className="text-foreground/50"
        />
      </span>
    );
  }
  const analysed = hasAnalysis || stage?.kind === "Done";
  return (
    <span title={analysed ? "Analysed" : "Not analysed"}>
      <ChartSpline
        size={ANALYSIS_ICON_SIZE_PX}
        className={analysed ? "text-primary" : "text-foreground/25"}
      />
    </span>
  );
};

interface GameActionsMenuProps {
  close: () => void;
  onOpen?: () => void;
  onAnalyze?: () => void;
  onExportPgn?: () => void;
  onDelete?: () => void;
}

const MENU_ITEM_CLASS =
  "flex items-center gap-2 w-full px-3 py-1.5 text-sm text-left hover:bg-white/10 cursor-pointer";
const MENU_ICON_SIZE_PX = 14;

// Delete is two-step: the row swaps to a confirm/cancel pair before it runs.
const GameActionsMenu = ({
  close,
  onOpen,
  onAnalyze,
  onExportPgn,
  onDelete,
}: GameActionsMenuProps) => {
  const [confirmingDelete, setConfirmingDelete] = useState(false);

  const run = (handler?: () => void) => () => {
    handler?.();
    close();
  };

  return (
    <div className="w-40 py-1 rounded-md bg-card border border-accent/60 shadow-lg">
      <button
        type="button"
        className={MENU_ITEM_CLASS}
        disabled={!onAnalyze}
        onClick={run(onAnalyze)}
      >
        <RotateCcw size={MENU_ICON_SIZE_PX} /> Analyze
      </button>
      <button
        type="button"
        className={MENU_ITEM_CLASS}
        disabled={!onOpen}
        onClick={run(onOpen)}
      >
        <Eye size={MENU_ICON_SIZE_PX} /> Open
      </button>
      <button
        type="button"
        className={MENU_ITEM_CLASS}
        disabled={!onExportPgn}
        onClick={run(onExportPgn)}
      >
        <Download size={MENU_ICON_SIZE_PX} /> Export PGN
      </button>

      {confirmingDelete ? (
        <div className="flex items-center justify-between gap-2 px-3 py-1.5 text-sm text-red-600 dark:text-red-400">
          <span>Delete?</span>
          <div className="flex items-center gap-1">
            <button
              type="button"
              aria-label="Confirm delete"
              className="rounded p-0.5 hover:bg-white/10"
              onClick={run(onDelete)}
            >
              <Check size={MENU_ICON_SIZE_PX} />
            </button>
            <button
              type="button"
              aria-label="Cancel delete"
              className="rounded p-0.5 text-foreground/60 hover:bg-white/10"
              onClick={() => setConfirmingDelete(false)}
            >
              <X size={MENU_ICON_SIZE_PX} />
            </button>
          </div>
        </div>
      ) : (
        <button
          type="button"
          className={`${MENU_ITEM_CLASS} text-red-600 dark:text-red-400`}
          disabled={!onDelete}
          onClick={() => setConfirmingDelete(true)}
        >
          <Trash2 size={MENU_ICON_SIZE_PX} /> Delete
        </button>
      )}
    </div>
  );
};

interface PlayerRowProps {
  color: PieceColor;
  name: string;
  elo: number;
}

const PlayerRow = ({ color, name, elo }: PlayerRowProps) => (
  <div className="flex items-center gap-2 min-w-0">
    <span
      className={`w-2.5 h-2.5 rounded-full shrink-0 ${PLAYER_DOT_CLASS[color]}`}
    />
    <span className="text-md text-foreground/90 truncate">
      {name} <span className="text-foreground/50">({elo})</span>
    </span>
  </div>
);

const GameCard = ({
  gameSummary,
  onOpen,
  onAnalyze,
  onExportPgn,
  onDelete,
}: GameCardProps) => {
  // An import with no clock has no mode, and shows no icon for one.
  const mode =
    gameSummary.time_control && modeForTimeControl(gameSummary.time_control);
  const analysisStage = useAnalysisStage(gameSummary.game_id);

  return (
    <div className="flex flex-col justify-between gap-2 px-4 py-3 w-full h-62 rounded-lg bg-card border border-border">
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-2 text-foreground/50">
          <span title={mode ?? undefined}>
            {mode && <ModeIcon mode={mode} />}
          </span>
          <AnalysisIndicator
            hasAnalysis={gameSummary.has_analysis}
            stage={analysisStage}
          />
          {gameSummary.source !== "koch" && (
            <span title={`Imported from ${gameSummary.source}`}>
              <Import size={ANALYSIS_ICON_SIZE_PX} />
            </span>
          )}
          {gameSummary.partial_import && (
            <span title="Imported partially — some moves couldn't be read">
              <TriangleAlert
                size={ANALYSIS_ICON_SIZE_PX}
                className="text-amber-500"
              />
            </span>
          )}
        </div>
        <Dropdown
          align="right"
          trigger={({ toggle }) => (
            <button
              type="button"
              aria-label="Game actions"
              onClick={toggle}
              className="p-1 rounded-md text-foreground/50 bg-accent/20 hover:text-foreground hover:bg-accent/50"
            >
              <MoreVertical size={MODE_ICON_SIZE_PX} />
            </button>
          )}
        >
          {(close) => (
            <GameActionsMenu
              close={close}
              onOpen={onOpen && (() => onOpen(gameSummary))}
              onAnalyze={onAnalyze && (() => onAnalyze(gameSummary))}
              onExportPgn={onExportPgn && (() => onExportPgn(gameSummary))}
              onDelete={onDelete && (() => onDelete(gameSummary))}
            />
          )}
        </Dropdown>
      </div>
      <CardSeparator />
      <PlayerRow
        color="white"
        name={gameSummary.white_player}
        elo={gameSummary.white_elo}
      />
      <PlayerRow
        color="black"
        name={gameSummary.black_player}
        elo={gameSummary.black_elo}
      />
      <CardSeparator />
      <div className="flex flex-row items-center gap-2">
        <ChessKing className="text-primary" />
        <span className="text-sm text-muted-foreground">
          {gameSummary.opening_name}
        </span>
      </div>
      <CardSeparator />
      <div className="flex justify-between items-center">
        <span
          className={`text-sm px-2 py-1 font-mono rounded-md ${getScoreCardStyle(gameSummary.human_score)}`}
        >
          {resultToken(gameSummary.result)}
        </span>
        <span className="text-sm text-foreground/60">
          {formatDatePlayed(gameSummary.date_played)}
        </span>
      </div>
    </div>
  );
};

export default GameCard;
