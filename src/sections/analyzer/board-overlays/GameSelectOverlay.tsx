import { useEffect, useState } from "react";
import type { GameSummary } from "../../../api/bindings/GameSummary";
import { getGames } from "../../../api/game";
import {
  formatDatePlayed,
  getScoreCardStyle,
  resultToken,
} from "../../../lib/gameSummary";
import { BOARD_PIXEL_SIZE } from "../../../components/chessboard/lib/constants";
import DialogCard, { WIDE_DIALOG_PX } from "../../../components/DialogCard";
import EmptyNote from "../../../components/EmptyNote";

// Stand-in for a date the game didn't record, same as the Game tab's.
const UNKNOWN = "—";

// Most of the board's height: a fixed height is what lets the list scroll
// inside the card, and leaving a margin keeps the board visible around it.
const PICKER_HEIGHT_PX = BOARD_PIXEL_SIZE * 0.8;

interface GameListCardProps {
  game: GameSummary;
  isCurrent: boolean;
  onPick: (gameId: number) => void;
}

// Two lines, each split to the edges: who played and when, then how it
// opened and how it ended. The left side truncates, so a long name can't
// push the right side off the card.
const GameListCard = ({ game, isCurrent, onPick }: GameListCardProps) => (
  <button
    type="button"
    aria-current={isCurrent}
    onClick={() => onPick(game.game_id)}
    className={`flex w-full cursor-pointer flex-col gap-1 rounded-md border-[1px] px-3 py-2 text-left transition-colors duration-150 hover:bg-primary/30 ${
      isCurrent
        ? "border-primary bg-primary/15"
        : "border-border bg-secondary/30"
    }`}
  >
    <div className="flex items-baseline justify-between gap-3">
      <span className="truncate text-sm text-card-foreground/90">
        {game.white_player} vs {game.black_player}
      </span>
      <span className="shrink-0 text-xs text-card-foreground/50">
        {formatDatePlayed(game.date_played) || UNKNOWN}
      </span>
    </div>
    <div className="flex items-baseline justify-between gap-3">
      <span className="truncate text-xs text-card-foreground/60">
        {game.opening_name ?? "Out of book"}
      </span>
      {/* History's result colors, at this line's size — tighter padding
          than the history card's chip, so the line doesn't grow taller. */}
      <span
        className={`shrink-0 rounded-md px-1.5 font-mono text-xs ${getScoreCardStyle(game.human_score)}`}
      >
        {resultToken(game.result)}
      </span>
    </div>
  </button>
);

interface GameSelectOverlayProps {
  // The game on screen, marked in the list. Null in the sandbox.
  currentGameId: number | null;
  onPick: (gameId: number) => void;
}

/**
 * Every saved game, to switch the analyzer to.
 *
 * Fetched each time it opens — it only exists while open — so a game
 * finished or imported since the last look is already in the list.
 */
const GameSelectOverlay = ({
  currentGameId,
  onPick,
}: GameSelectOverlayProps) => {
  // Null until the fetch lands, so "no games" only shows once it's true.
  const [games, setGames] = useState<GameSummary[] | null>(null);

  useEffect(() => {
    getGames().then(setGames).catch(console.error);
  }, []);

  return (
    <DialogCard
      title="Choose a game"
      widthPx={WIDE_DIALOG_PX}
      heightPx={PICKER_HEIGHT_PX}
    >
      {games?.length === 0 ? (
        <EmptyNote>No saved games yet</EmptyNote>
      ) : (
        <ul className="flex min-h-0 w-full flex-1 flex-col gap-2 overflow-y-auto">
          {games?.map((game) => (
            <li key={game.game_id}>
              <GameListCard
                game={game}
                isCurrent={game.game_id === currentGameId}
                onPick={onPick}
              />
            </li>
          ))}
        </ul>
      )}
    </DialogCard>
  );
};

export default GameSelectOverlay;
