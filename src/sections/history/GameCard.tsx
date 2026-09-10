import { useState, type ReactNode } from "react";
import {
  Flame,
  Zap,
  Clock,
  ChessKnight,
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
} from "lucide-react";
import { GameSummary } from "../../api/bindings/GameSummary";
import { PieceColor } from "../../api/bindings/PieceColor";
import { AnalysisStage } from "../../api/bindings/AnalysisStage";
import { useAnalysisStage } from "../../app/AppContext";
import { MODE_TIME_CONTROL, type GameMode } from "../play/GameContext";
import Dropdown from "../../components/Dropdown";

interface GameCardProps {
  gameSummary: GameSummary;
  onOpen?: (game: GameSummary) => void;
  onAnalyze?: (game: GameSummary) => void;
  onExportPgn?: (game: GameSummary) => void;
  onDelete?: (game: GameSummary) => void;
}

const MODE_ICON_SIZE = 18;

const MODE_ICON: Record<GameMode, ReactNode> = {
  Bullet: <Flame size={MODE_ICON_SIZE} className="text-red-500" />,
  Blitz: <Zap size={MODE_ICON_SIZE} className="text-amber-500" />,
  Rapid: <Clock size={MODE_ICON_SIZE} className="text-sky-600" />,
  Classical: <ChessKnight size={MODE_ICON_SIZE} className="text-violet-400" />,
};

const MODE_ORDER: GameMode[] = ["Bullet", "Blitz", "Rapid", "Classical"];

const modeForTimeControl = (timeControl: string | null): GameMode | null => {
  if (!timeControl) return null;
  const initialMs = parseInt(timeControl, 10);
  if (Number.isNaN(initialMs)) return null;
  return (
    MODE_ORDER.find((mode) => initialMs <= MODE_TIME_CONTROL[mode].initial_ms) ??
    "Classical"
  );
};

// SQLite's `datetime('now')` is UTC "YYYY-MM-DD HH:MM:SS" with no zone marker.
const formatDatePlayed = (datePlayed: string | null): string => {
  if (!datePlayed) return "";
  const date = new Date(`${datePlayed.replace(" ", "T")}Z`);
  if (isNaN(date.getTime())) return datePlayed;
  return date.toLocaleDateString(undefined, {
    year: "numeric",
    month: "short",
    day: "numeric",
  });
};

const PLAYER_DOT_CLASS: Record<PieceColor, string> = {
  white: "bg-neutral-100 border border-white",
  black: "bg-neutral-900 border border-white",
};
const WIN_SCORE_CARD_COLOR =
  "bg-emerald-500/15 text-emerald-600 dark:text-emerald-400 border border-emerald-500/30"
const LOSE_SCORE_CARD_COLOR =
  "bg-red-500/15 text-red-600 dark:text-red-400 border border-red-500/30"
const DRAW_SCORE_CARD_COLOR =
  "bg-muted/40 text-muted-foreground border border-border"

// A null `human_color` (self-play, imports) is deliberately neutral, like a draw.
const getScoreCardStyle = (game: GameSummary): string => {
  if (game.human_color === null || game.result === "1/2-1/2") {
    return DRAW_SCORE_CARD_COLOR
  }
  const humanWon =
    (game.result === "1-0" && game.human_color === "white") ||
    (game.result === "0-1" && game.human_color === "black")
  return humanWon ? WIN_SCORE_CARD_COLOR : LOSE_SCORE_CARD_COLOR
}
const CardSeparator = () => <hr className="border-border my-[1px]" />

const ANALYSIS_ICON_SIZE = 18

const AnalysisRing = ({ percent }: { percent: number }) => {
  const radius = 8
  const circumference = 2 * Math.PI * radius
  return (
    <span
      title={`Analysing… ${percent}%`}
      className="relative inline-flex animate-pulse items-center justify-center"
      style={{ width: ANALYSIS_ICON_SIZE, height: ANALYSIS_ICON_SIZE }}
    >
      <svg
        width={ANALYSIS_ICON_SIZE}
        height={ANALYSIS_ICON_SIZE}
        className="-rotate-90"
      >
        <circle
          cx={ANALYSIS_ICON_SIZE / 2}
          cy={ANALYSIS_ICON_SIZE / 2}
          r={radius}
          fill="none"
          strokeWidth={2}
          className="stroke-border"
        />
        <circle
          cx={ANALYSIS_ICON_SIZE / 2}
          cy={ANALYSIS_ICON_SIZE / 2}
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
  )
}

// A `Done` event counts as analysed straight away, before the list refetches.
const AnalysisIndicator = ({
  hasAnalysis,
  stage,
}: {
  hasAnalysis: boolean
  stage: AnalysisStage | undefined
}) => {
  if (stage?.kind === "Running") return <AnalysisRing percent={stage.percent} />
  if (stage?.kind === "Queued") {
    return (
      <span title="Analysis queued">
        <Hourglass size={ANALYSIS_ICON_SIZE} className="text-foreground/50" />
      </span>
    )
  }
  const analysed = hasAnalysis || stage?.kind === "Done"
  return (
    <span title={analysed ? "Analysed" : "Not analysed"}>
      <ChartSpline
        size={ANALYSIS_ICON_SIZE}
        className={analysed ? "text-primary" : "text-foreground/25"}
      />
    </span>
  )
}

interface GameActionsMenuProps {
  close: () => void;
  onOpen?: () => void;
  onAnalyze?: () => void;
  onExportPgn?: () => void;
  onDelete?: () => void;
}

const MENU_ITEM_CLASS =
  "flex items-center gap-2 w-full px-3 py-1.5 text-sm text-left hover:bg-white/10 cursor-pointer"
const MENU_ICONS_SIZE = 14

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
        <RotateCcw size={MENU_ICONS_SIZE} /> Analyze
      </button>
      <button
        type="button"
        className={MENU_ITEM_CLASS}
        disabled={!onOpen}
        onClick={run(onOpen)}
      >
        <Eye size={MENU_ICONS_SIZE} /> Open
      </button>
      <button
        type="button"
        className={MENU_ITEM_CLASS}
        disabled={!onExportPgn}
        onClick={run(onExportPgn)}
      >
        <Download size={MENU_ICONS_SIZE} /> Export PGN
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
              <Check size={MENU_ICONS_SIZE} />
            </button>
            <button
              type="button"
              aria-label="Cancel delete"
              className="rounded p-0.5 text-foreground/60 hover:bg-white/10"
              onClick={() => setConfirmingDelete(false)}
            >
              <X size={MENU_ICONS_SIZE} />
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
          <Trash2 size={MENU_ICONS_SIZE} /> Delete
        </button>
      )}
    </div>
  );
}

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
  const mode = modeForTimeControl(gameSummary.time_control);
  const analysisStage = useAnalysisStage(gameSummary.game_id);

  return (
    <div className="flex flex-col justify-between gap-2 px-4 py-3 w-full h-62 rounded-lg bg-card border border-border">
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-2 text-foreground/50">
          <span title={mode ?? undefined}>{mode && MODE_ICON[mode]}</span>
          <AnalysisIndicator
            hasAnalysis={gameSummary.has_analysis}
            stage={analysisStage}
          />
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
              <MoreVertical size={MODE_ICON_SIZE} />
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
        <span className="text-sm text-muted-foreground">{gameSummary.opening_name}</span>
      </div>
      <CardSeparator />
      <div className="flex justify-between items-center">
        <span
          className={`text-sm px-2 py-1 font-mono rounded-md ${getScoreCardStyle(gameSummary)}`}
        >
          {gameSummary.result}
        </span>
        <span className="text-sm text-foreground/60">{formatDatePlayed(gameSummary.date_played)}</span>
      </div>
    </div>
  );
};

export default GameCard;
