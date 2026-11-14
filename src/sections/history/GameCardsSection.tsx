import { useEffect, useRef, useState } from "react";
import { useNavigate } from "react-router-dom";
import { getGames, countGames, deleteGame, analyzeGame } from "../../api/game";
import type { GameSummary } from "../../api/bindings/GameSummary";
import type { PieceColor } from "../../api/bindings/PieceColor";
import GameCard from "./GameCard";
import AnalyzeSidePopup from "./AnalyzeSidePopup";
import { useAnalyzerContext } from "../analyzer/AnalyzerContext";
import { HistoryIcon, RefreshCcw, Swords, Upload } from "lucide-react";
import { OutlineButton, PrimaryButton } from "../../components/buttons";
import EmptyNote from "../../components/EmptyNote";
import EmptyState from "../../components/EmptyState";
import Pager from "../../components/Pager";

// Ten rows of four cards.
const PAGE_SIZE = 40;
const ACTION_ICON_SIZE_PX = 16;

interface GamePage {
  index: number;
  games: GameSummary[];
  pageCount: number;
}

interface NoGamesProps {
  syncing: boolean;
  onSync: () => void;
  onImport: () => void;
}

// A history with no games at all: the three ways to get some.
const NoGames = ({ syncing, onSync, onImport }: NoGamesProps) => {
  const navigate = useNavigate();
  return (
    <EmptyState
      icon={HistoryIcon}
      message="Your games show up here: play one against the engine, import a PGN, or sync your chess.com games."
    >
      <PrimaryButton onClick={() => navigate("/play")}>
        <Swords size={ACTION_ICON_SIZE_PX} /> Play a game
      </PrimaryButton>
      <OutlineButton onClick={onImport}>
        <Upload size={ACTION_ICON_SIZE_PX} /> Import PGN
      </OutlineButton>
      <OutlineButton onClick={onSync} disabled={syncing}>
        <RefreshCcw
          size={ACTION_ICON_SIZE_PX}
          className={syncing ? "animate-spin" : undefined}
        />
        Sync chess.com
      </OutlineButton>
    </EmptyState>
  );
};

interface GameCardsSectionProps extends NoGamesProps {
  search: string;
  // The first day to include, as "YYYY-MM-DD"; none for all time.
  playedFrom?: string;
  // Bumped by the toolbar after an import, so the list fetches again.
  reloadKey?: number;
}

const GameCardsSection = ({
  search,
  playedFrom,
  reloadKey,
  syncing,
  onSync,
  onImport,
}: GameCardsSectionProps) => {
  const [pageIndex, setPageIndex] = useState(0);
  // The page on screen. Kept while the next one loads, so the cards don't
  // blink out between pages.
  const [page, setPage] = useState<GamePage | null>(null);
  // Bumped after a delete, so the page refills from the one after it.
  const [deletions, setDeletions] = useState(0);
  const [sideNeeded, setSideNeeded] = useState<GameSummary | null>(null);
  const top = useRef<HTMLDivElement>(null);
  const { openGame } = useAnalyzerContext();
  const navigate = useNavigate();

  useEffect(() => {
    let stale = false;
    const filter = { search, played_from: playedFrom };
    Promise.all([
      getGames({ ...filter, limit: PAGE_SIZE, offset: pageIndex * PAGE_SIZE }),
      countGames(filter),
    ])
      .then(([games, total]) => {
        if (stale) return;
        const pageCount = Math.ceil(total / PAGE_SIZE);
        // Deleting the last game on the last page leaves nothing to show:
        // step back to the page before.
        if (games.length === 0 && pageIndex > 0) {
          setPageIndex(Math.max(0, pageCount - 1));
          return;
        }
        setPage({ index: pageIndex, games, pageCount });
      })
      .catch(console.error);
    return () => {
      stale = true;
    };
  }, [search, playedFrom, pageIndex, reloadKey, deletions]);

  const goToPage = (index: number) => {
    setPageIndex(index);
    top.current?.scrollIntoView();
  };

  const handleDelete = (game: GameSummary) => {
    deleteGame(game.game_id)
      .then(() => setDeletions((count) => count + 1))
      .catch(console.error);
  };

  // Fire-and-forget — the analysis runs on the backend queue. A koch game
  // is graded from the side you played, and a chess.com game from the side
  // the sync found you on. A pasted PGN has no inherent side, so you pick
  // one every time (defaulting to the last pick).
  const handleAnalyze = (game: GameSummary) => {
    if (game.source === "pgn" || game.human_color === null) {
      setSideNeeded(game);
      return;
    }
    analyzeGame(game.game_id).catch(console.error);
  };

  // Steps through the game in the analyzer — works whether or not it's
  // been analysed; an unanalysed game just has no evals or move grades.
  const handleOpen = (game: GameSummary) => {
    openGame(game.game_id);
    navigate(`/analysis/${game.game_id}`);
  };

  const handlePickSide = (color: PieceColor) => {
    if (sideNeeded) analyzeGame(sideNeeded.game_id, color).catch(console.error);
    setSideNeeded(null);
  };

  // Nothing until the first page arrives, so a full history doesn't flash
  // the empty state on its way in.
  const cards = () => {
    if (page === null) return null;
    if (page.games.length === 0) {
      return search || playedFrom ? (
        <div className="px-6 py-8 text-center">
          <EmptyNote>No games match this search</EmptyNote>
        </div>
      ) : (
        <NoGames syncing={syncing} onSync={onSync} onImport={onImport} />
      );
    }
    return (
      <div className="flex flex-wrap gap-3 px-6 py-4">
        {page.games.map((game) => (
          <div
            key={game.game_id}
            className="w-full max-w-[calc(25%-0.6875rem)]"
          >
            <GameCard
              gameSummary={game}
              onOpen={handleOpen}
              onAnalyze={handleAnalyze}
              onDelete={handleDelete}
            />
          </div>
        ))}
      </div>
    );
  };

  return (
    <div ref={top}>
      {cards()}
      {page && (
        <Pager
          pageIndex={page.index}
          pageCount={page.pageCount}
          onPageChange={goToPage}
        />
      )}
      <AnalyzeSidePopup
        game={sideNeeded}
        onPick={handlePickSide}
        onClose={() => setSideNeeded(null)}
      />
    </div>
  );
};

export default GameCardsSection;
