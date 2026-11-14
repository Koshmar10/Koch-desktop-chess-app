import { useEffect, useState } from "react";
import GameCardsSection from "./GameCardsSection";
import GamePlaytimeCalendar from "./GamePlaytimeCalendar";
import HistoryToolbar from "./HistoryToolbar";
import { periodStart, type Period } from "../../lib/period";

// Long enough that typing a name searches once, not once per letter.
const SEARCH_DEBOUNCE_MS = 300;

const History = () => {
  const [reloadKey, setReloadKey] = useState(0);
  const [search, setSearch] = useState("");
  const [searched, setSearched] = useState("");
  const [period, setPeriod] = useState<Period>("All Time");

  useEffect(() => {
    const timer = setTimeout(() => setSearched(search), SEARCH_DEBOUNCE_MS);
    return () => clearTimeout(timer);
  }, [search]);

  const playedFrom = periodStart(period);

  return (
    <div className="w-full h-full flex flex-col">
      <GamePlaytimeCalendar />
      <HistoryToolbar
        search={search}
        onSearchChange={setSearch}
        period={period}
        onPeriodChange={setPeriod}
        onImported={() => setReloadKey((key) => key + 1)}
      />
      {/* Keyed by the filter, so a new search starts again from page 1. */}
      <GameCardsSection
        key={`${searched}|${period}`}
        search={searched}
        playedFrom={playedFrom}
        reloadKey={reloadKey}
      />
    </div>
  );
};

export default History;
