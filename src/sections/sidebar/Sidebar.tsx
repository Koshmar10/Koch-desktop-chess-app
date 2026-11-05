import { useEffect, useEffectEvent } from "react";
import { useNavigate } from "react-router-dom";
import {
  Home,
  Play,
  LineChart,
  History,
  Puzzle,
  Settings,
  Clapperboard,
} from "lucide-react";
import kochLogo from "../../assets/koch-logo.svg";
import SidebarButton from "./SidebarButton";
import { useAnalyzerContext } from "../analyzer/AnalyzerContext";
import {
  commandForKey,
  shortcutLabel,
  type PaletteCommand,
  type Shortcut,
} from "../../lib/commandPalette";

// In the order shown; Ctrl+1 is the first.
const SIDEBAR_ITEMS = [
  { to: "/", label: "Home", icon: Home },
  { to: "/play", label: "Play", icon: Play },
  { to: "/analysis", label: "Analysis", icon: LineChart },
  { to: "/history", label: "History", icon: History },
  { to: "/puzzle", label: "Puzzles", icon: Puzzle },
  { to: "/moments", label: "Moments", icon: Clapperboard },
];

const SETTINGS_PATH = "/settings";
// The common desktop key for an app's settings. It types nothing, so it
// works from a text field too, as the screen keys do.
const SETTINGS_SHORTCUT: Shortcut = { key: ",", ctrl: true, whileTyping: true };

// Ctrl and the screen's place in the sidebar, counting from 1.
const screenShortcut = (index: number): Shortcut => ({
  key: String(index + 1),
  ctrl: true,
  whileTyping: true,
});

const LogoSection = () => (
  <div className="flex justify-center items-center pt-8 border-b-[1px] border-sidebar-border pb-6">
    <img src={kochLogo} alt="Koch" className="size-12" />
  </div>
);

// Where each screen's button goes: the analysis one back to the last game
// opened, if there was one, rather than always to the sandbox.
const destinationOf = (to: string, loadedGameId: number | null) =>
  to === "/analysis" && loadedGameId !== null ? `${to}/${loadedGameId}` : to;

const ButtonSection = ({ loadedGameId }: { loadedGameId: number | null }) => (
  <div className="h-full flex flex-col px-2 gap-4 items-center">
    {SIDEBAR_ITEMS.map(({ to, label, icon }, index) => {
      const destination = destinationOf(to, loadedGameId);
      return (
        <SidebarButton
          key={destination}
          to={destination}
          icon={icon}
          title={`${label} (${shortcutLabel(screenShortcut(index))})`}
        />
      );
    })}
  </div>
);

const Sidebar = () => {
  const { loadedGameId } = useAnalyzerContext();
  const navigate = useNavigate();

  // The sidebar's buttons by keyboard, from any screen. An effect event,
  // so the listener added once reads the latest game to go back to.
  const onKeyDown = useEffectEvent((event: KeyboardEvent) => {
    const screens: PaletteCommand[] = [
      ...SIDEBAR_ITEMS.map(({ to, label }, index) => ({
        id: `screen:${to}`,
        label,
        shortcuts: [screenShortcut(index)],
        run: () => navigate(destinationOf(to, loadedGameId)),
      })),
      {
        id: "screen:settings",
        label: "Settings",
        shortcuts: [SETTINGS_SHORTCUT],
        run: () => navigate(SETTINGS_PATH),
      },
    ];
    // Every one of these types nothing, so a text field doesn't stop them.
    const command = commandForKey(screens, event, true);
    if (!command) return;
    event.preventDefault();
    command.run();
  });

  useEffect(() => {
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, []);

  return (
    <div className="flex flex-col w-20 shrink-0 border-r-[1px] border-sidebar-border gap-6 bg-sidebar">
      <LogoSection />
      <ButtonSection loadedGameId={loadedGameId} />
      <div className="flex flex-row justify-center py-4 items-center border-t-[1px] border-sidebar-border">
        <SidebarButton
          to={SETTINGS_PATH}
          icon={Settings}
          title={`Settings (${shortcutLabel(SETTINGS_SHORTCUT)})`}
        />
      </div>
    </div>
  );
};

export default Sidebar;
