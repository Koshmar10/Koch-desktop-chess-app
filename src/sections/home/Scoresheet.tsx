import { ArrowRight } from "lucide-react";
import { Link } from "react-router-dom";
import type { GameSummary } from "../../api/bindings/GameSummary";
import type { GameScore } from "../../api/bindings/GameScore";
import EmptyNote from "../../components/EmptyNote";
import { formatDatePlayed, resultToken } from "../../lib/gameSummary";
import { latestGames } from "../../lib/homeStats";
import HomeBlock from "./HomeBlock";

const SCORESHEET_ROWS = 5;

const SCORE_TEXT_CLASS: Record<GameScore, string> = {
  Win: "text-emerald-500",
  Loss: "text-red-400",
  Draw: "text-foreground/60",
};

interface ScoresheetProps {
  games: GameSummary[];
}

const Scoresheet = ({ games }: ScoresheetProps) => {
  const rows = latestGames(games, SCORESHEET_ROWS);

  return (
    <HomeBlock title="The scoresheet" subtitle="Your latest games.">
      {rows.length === 0 ? (
        <EmptyNote>No games yet</EmptyNote>
      ) : (
        <ul className="flex flex-col">
          {rows.map((game) => (
            <li
              key={game.game_id}
              className="flex items-center gap-3 border-b-[1px] border-dashed border-border/60 py-2 font-mono text-xs"
            >
              <span className="w-24 shrink-0 text-foreground/50">
                {formatDatePlayed(game.date_played)}
              </span>
              <span className="min-w-0 flex-1 truncate text-foreground">
                {game.opening_name ?? "—"}
              </span>
              <span className={SCORE_TEXT_CLASS[game.human_score ?? "Draw"]}>
                {resultToken(game.result)}
              </span>
            </li>
          ))}
        </ul>
      )}
      <Link
        to="/history"
        className="flex items-center gap-1 self-start text-xs text-primary hover:underline"
      >
        All games <ArrowRight size={12} />
      </Link>
    </HomeBlock>
  );
};

export default Scoresheet;
