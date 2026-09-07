import { Route, Routes } from "react-router-dom";
import "./App.css";
import Sidebar from "./sections/sidebar/Sidebar";
import Home from "./sections/home/Home";
import Play from "./sections/play/Play";
import { GameProvider } from "./sections/play/GameProvider";
import Analysis from "./sections/Analysis";
import History from "./sections/history/History";
import Puzzle from "./sections/Puzzle";
import Settings from "./sections/settings/Settings";

function App() {
  return (
    // GameProvider lives here, not inside the /play route itself: it
    // listens for backend events (engine replies, analysis progress/
    // completion) that keep arriving after a game ends regardless of what
    // page is open. Scoping it to /play meant navigating away tore down
    // those listeners mid-analysis and silently dropped everything the
    // backend sent from that point on.
    <GameProvider>
      <div className="flex flex-row w-[100vw] h-[100vh]">
        <Sidebar />
        <div className="flex-1 min-w-0 overflow-x-auto">
          <Routes>
            <Route path="/" element={<Home />} />
            <Route path="/play" element={<Play />} />
            <Route path="/analysis" element={<Analysis />} />
            <Route path="/history" element={<History />} />
            <Route path="/puzzle" element={<Puzzle />} />
            <Route path="/settings" element={<Settings />} />
          </Routes>
        </div>
      </div>
    </GameProvider>
  );
}

export default App;
