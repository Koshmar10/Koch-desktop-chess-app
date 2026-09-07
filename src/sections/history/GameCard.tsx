import { type ReactNode } from "react";
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
} from "lucide-react";
import { GameSummary } from "../../api/bindings/GameSummary";
import { PieceColor } from "../../api/bindings/PieceColor";
import { MODE_TIME_CONTROL, type GameMode } from "../play/GameContext";
import Dropdown from "../../components/Dropdown";

interface GameCardProps {
  gameSummary: GameSummary;
  onOpen?: (game: GameSummary) => void;
  onExportPgn?: (game: GameSummary) => void;
  onDelete?: (game: GameSummary) => void;
}

const MODE_ICON_SIZE = 18;

// Same hue per mode the old app's time-control tags used (Bullet red,
// Blitz orange, Rapid blue, Classical purple) — just carried by the icon's
// own color now instead of a colored badge chip.
const MODE_ICON: Record<GameMode, ReactNode> = {
  Bullet: <Flame size={MODE_ICON_SIZE} className="text-red-500" />,
  Blitz: <Zap size={MODE_ICON_SIZE} className="text-amber-500" />,
  Rapid: <Clock size={MODE_ICON_SIZE} className="text-sky-600" />,
  Classical: <ChessKnight size={MODE_ICON_SIZE} className="text-violet-400" />,
};

// Ascending by initial_ms — the same presets PlayControls offers. A saved
// game's time_control gets bucketed into the first preset it doesn't
// exceed, falling through to Classical for anything longer (or anything
// off-preset, e.g. a future chess.com import).
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

// `date_played` comes off SQLite's `datetime('now')` as "YYYY-MM-DD
// HH:MM:SS", UTC but with no timezone marker — reshape it into something
// `Date` parses reliably before falling back to the raw string.
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

// `human_color` is stored lowercase ('white'/'black', see GameService::
// save_game) and is null for a game with no human side (self-play, or a
// future import) — neither "win" nor "lose" means anything there, so that
// and an actual draw both get the neutral color rather than falling
// through to whichever check happens to be last.
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

interface GameActionsMenuProps {
  close: () => void;
  onOpen?: () => void;
  onExportPgn?: () => void;
  onDelete?: () => void;
}

const MENU_ITEM_CLASS =
  "flex items-center gap-2 w-full px-3 py-1.5 text-sm text-left hover:bg-white/10 cursor-pointer"
const MENU_ICONS_SIZE = 14

// The card only ever renders the handlers its parent passes — an action
// with no handler is shown disabled rather than hidden, so the menu's
// shape stays stable across the grid.
const GameActionsMenu = ({
  close,
  onOpen,
  onExportPgn,
  onDelete,
}: GameActionsMenuProps) => {
  const run = (handler?: () => void) => () => {
    handler?.();
    close();
  };
  return (
    <div className="w-40 py-1 rounded-md bg-card border border-accent/60 shadow-lg">
      <button
        type="button"
        className={MENU_ITEM_CLASS}
        disabled={!onExportPgn}
        onClick={run(onExportPgn)}
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

      <button
        type="button"
        className={`${MENU_ITEM_CLASS} text-red-600 dark:text-red-400`}
        disabled={!onDelete}
        onClick={run(onDelete)}
      >
        <Trash2 size={MENU_ICONS_SIZE} /> Delete
      </button>
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
  onExportPgn,
  onDelete,
}: GameCardProps) => {
  const mode = modeForTimeControl(gameSummary.time_control);

  return (
    <div className="flex flex-col gap-2 px-4 py-3 w-full rounded-lg bg-card border border-border">
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-2 text-foreground/50">
          <span title={mode ?? undefined}>{mode && MODE_ICON[mode]}</span>
          <span
            title={gameSummary.has_analysis ? "Analysed" : "Not analysed"}
          >
            <ChartSpline
              size={MODE_ICON_SIZE}
              className={
                gameSummary.has_analysis ? "text-primary" : "text-foreground/25"
              }
            />
          </span>
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
