import { Route, Routes } from "react-router-dom";
import "./App.css";
import Sidebar from "./sections/sidebar/Sidebar";
import Home from "./sections/home/Home";
import Play from "./sections/play/Play";
import Analyzer from "./sections/analyzer/Analyzer";
import History from "./sections/history/History";
import Puzzle from "./sections/puzzle/Puzzle";
import Settings from "./sections/settings/Settings";

const App = () => {
  return (
    <div className="flex flex-row w-[100vw] h-[100vh]">
      <Sidebar />
      <div className="flex-1 min-w-0 overflow-x-auto">
        <Routes>
          <Route path="/" element={<Home />} />
          <Route path="/play" element={<Play />} />
          {/* No id is the sandbox; with one, a saved game to step through. */}
          <Route path="/analysis/:gameId?" element={<Analyzer />} />
          <Route path="/history" element={<History />} />
          <Route path="/puzzle" element={<Puzzle />} />
          <Route path="/settings" element={<Settings />} />
        </Routes>
      </div>
    </div>
  );
};

export default App;
