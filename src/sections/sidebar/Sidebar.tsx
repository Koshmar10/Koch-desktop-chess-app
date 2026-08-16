import { Home, Play, LineChart, History, Puzzle, Settings } from "lucide-react";
import SidebarButton from "./SidebarButton";

const SIDEBAR_BUTTON_SIZE_PX = 24

const SIDEBAR_ITEMS = [
  { to: "/", icon: <Home size={SIDEBAR_BUTTON_SIZE_PX} /> },
  { to: "/play", icon: <Play size={SIDEBAR_BUTTON_SIZE_PX} /> },
  { to: "/analysis", icon: <LineChart size={SIDEBAR_BUTTON_SIZE_PX} /> },
  { to: "/history", icon: <History size={SIDEBAR_BUTTON_SIZE_PX} /> },
  { to: "/puzzle", icon: <Puzzle size={SIDEBAR_BUTTON_SIZE_PX} /> },
  { to: "/settings", icon: <Settings size={SIDEBAR_BUTTON_SIZE_PX} /> },
];

const LogoSection = () => (
  <div className="flex justify-center items-center pt-8 border-b-[1px] border-sidebar-border pb-6">
    <span className="w-fit h-fit px-5 py-3 rounded-lg text-sidebar-foreground bg-accent text-xl">
      K
    </span>
  </div>
);

const ButtonSection = () => (
  <div className="h-full flex flex-col px-2 gap-4 items-center">
    {SIDEBAR_ITEMS.map(({ to, icon }) => (
      <SidebarButton key={to} to={to} icon={icon} />
    ))}
  </div>
);

const Sidebar = () => {
  return (
    <div className="flex flex-col w-[5%] border-r-[1px] border-sidebar-border gap-6 bg-sidebar-dark">
      <LogoSection />
      <ButtonSection />
    </div>
  );
};

export default Sidebar;
