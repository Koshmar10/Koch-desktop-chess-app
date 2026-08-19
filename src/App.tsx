import { Route, Routes } from "react-router-dom";
import "./App.css";
import Sidebar from "./sections/sidebar/Sidebar";
import Home from "./sections/home/Home";
import Play from "./sections/Play";
import Analysis from "./sections/Analysis";
import History from "./sections/history/History";
import Puzzle from "./sections/Puzzle";
import Settings from "./sections/Settings";

function App() {
  return (
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
  );
}

export default App;
