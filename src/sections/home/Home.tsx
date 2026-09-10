import { useEffect, useState } from "react";
import HomeCard from "./HomeCard";
import { STAT_CARDS } from "./mock";
import { randomChessQuote } from "./quotes";
import { getPlayerStats } from "../../api/stats";
import { getAppSettings } from "../../api/settings";
import { PlayerStats } from "../../api/bindings/PlayerStats";

const GreetMessage = () => {
  // Picked once per mount — lazy state, so it isn't re-rolled on re-render.
  const [{ quote, name }] = useState(randomChessQuote);
  const [playerName, setPlayerName] = useState<string | null>(null);
  useEffect(() => {
    getAppSettings()
      .then((s) => setPlayerName(s?.koch_username ?? null))
      .catch(console.error);
  }, []);

  return (
    <>
      <h1 className="text-3xl mt-6 mb-6 text-foreground">
        Welcome back, {playerName || "player"}
      </h1>
      <div className="text-md text-center text-foreground/60 italic self-center max-w-[60ch]">
        “{quote}”{" "}
        <span className="not-italic text-foreground/40">— {name}</span>
      </div>
    </>
  );
};

const StatCardsSection = () => {
  const [stats, setStats] = useState<PlayerStats | null>(null);
  useEffect(() => {
    getPlayerStats().then(setStats).catch(console.error);
  }, []);

  return (
    <div className="grid grid-cols-4 grid-rows-4 gap-4 w-[65%] h-[50rem] pt-20 px-4">
      {STAT_CARDS.map(({ format, ...card }) => (
        <HomeCard
          key={card.label}
          {...card}
          value={stats ? format(stats) : "—"}
        />
      ))}
    </div>
  );
};

const Home = () => {
  return (
    <div className="flex flex-col justify-start items-center w-full">
      <GreetMessage />
      <StatCardsSection />
    </div>
  );
};

export default Home;
