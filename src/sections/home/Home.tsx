import { useEffect, useState } from "react";
import { getGames } from "../../api/game";
import { getAppSettings } from "../../api/settings";
import { getPlayerStats, getRatingHistory } from "../../api/stats";
import type { GameSummary } from "../../api/bindings/GameSummary";
import type { PlayerStats } from "../../api/bindings/PlayerStats";
import type { RatingPoint } from "../../api/bindings/RatingPoint";
import AtTheBoard from "./AtTheBoard";
import FormGuide from "./FormGuide";
import OpeningLines from "./OpeningLines";
import PuzzleRecord from "./PuzzleRecord";
import RatingCard from "./RatingCard";
import Scoresheet from "./Scoresheet";
import WeekMoments from "./WeekMoments";
import YourArmy from "./YourArmy";
import { randomChessQuote } from "./quotes";

const Masthead = ({
  playerName,
  gameCount,
}: {
  playerName: string | null;
  gameCount: number;
}) => (
  <header className="flex items-center justify-between border-b-[1px] border-border/60 pb-2 text-[11px] text-foreground/50">
    <span>Koch · {playerName ?? "Player"}'s edition</span>
    <span>
      {new Date().toLocaleDateString(undefined, { dateStyle: "full" })}
    </span>
    <span>{gameCount} games on record</span>
  </header>
);

const Quote = () => {
  // Picked once per mount — lazy state, so it isn't re-rolled on re-render.
  const [{ quote, name }] = useState(randomChessQuote);

  return (
    <p className="max-w-[60ch] text-2xl text-foreground/60 italic">
      “{quote}” <span className="not-italic text-foreground/40">— {name}</span>
    </p>
  );
};

const Home = () => {
  const [stats, setStats] = useState<PlayerStats | null>(null);
  const [ratingHistory, setRatingHistory] = useState<RatingPoint[]>([]);
  const [games, setGames] = useState<GameSummary[]>([]);
  const [playerName, setPlayerName] = useState<string | null>(null);

  useEffect(() => {
    getPlayerStats().then(setStats).catch(console.error);
    getRatingHistory().then(setRatingHistory).catch(console.error);
    getGames().then(setGames).catch(console.error);
    getAppSettings()
      .then((settings) => setPlayerName(settings?.koch_username ?? null))
      .catch(console.error);
  }, []);

  return (
    <div className="mx-auto flex w-full max-w-6xl flex-col gap-6 px-8 py-6">
      <Masthead playerName={playerName} gameCount={games.length} />

      <section className="flex items-start justify-between gap-8">
        <Quote />
        <RatingCard rating={stats?.current_rating} history={ratingHistory} />
      </section>

      <div className="grid grid-cols-3 gap-6">
        <FormGuide />
        <YourArmy />
        <AtTheBoard stats={stats} />
      </div>

      <PuzzleRecord />

      <WeekMoments />

      <div className="grid grid-cols-2 gap-6">
        <Scoresheet games={games} />
        <OpeningLines games={games} />
      </div>
    </div>
  );
};

export default Home;
