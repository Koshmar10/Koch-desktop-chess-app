import GameCardsSection from "./GameCardsSection";
import ActivityChart from "./GamePlaytimeCalendar";
import HistoryToolbar from "./HistoryToolbar";

const History = () => {
  return (
    <div className="w-full h-full flex flex-col">
      <ActivityChart />
      <HistoryToolbar />
      <GameCardsSection />
    </div>
  );
};

export default History;
