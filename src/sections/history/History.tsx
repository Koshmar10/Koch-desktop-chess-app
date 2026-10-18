import { useState } from "react";
import GameCardsSection from "./GameCardsSection";
import GamePlaytimeCalendar from "./GamePlaytimeCalendar";
import HistoryToolbar from "./HistoryToolbar";

const History = () => {
  const [reloadKey, setReloadKey] = useState(0);

  return (
    <div className="w-full h-full flex flex-col">
      <GamePlaytimeCalendar />
      <HistoryToolbar onImported={() => setReloadKey((key) => key + 1)} />
      <GameCardsSection reloadKey={reloadKey} />
    </div>
  );
};

export default History;
