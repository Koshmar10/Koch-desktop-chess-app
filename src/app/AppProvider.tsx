import { useEffect, useState, type ReactNode } from "react";
import { AppContext, type Theme } from "./AppContext";
import { useAppEvent } from "./useAppEvent";
import type { AnalysisStage } from "../api/bindings/AnalysisStage";

const THEME_STORAGE_KEY = "koch-theme";

// Resolve the theme once on mount: the persisted choice if there is one,
// otherwise the OS preference for the very first run. App.css drives every
// token off a single `.dark` class on <html> (see its @custom-variant), so
// applying a theme later is just toggling that one class.
const initialTheme = (): Theme => {
  const stored = localStorage.getItem(THEME_STORAGE_KEY);
  if (stored === "light" || stored === "dark") return stored;
  return window.matchMedia("(prefers-color-scheme: dark)").matches
    ? "dark"
    : "light";
};

interface AppProviderProps {
  children: ReactNode;
}

// State that outlives any one route or game — theme and per-game analysis
// status. GameProvider nests inside this, not the other way round.
export function AppProvider({ children }: AppProviderProps) {
  const [theme, setTheme] = useState<Theme>(initialTheme);
  const [analysisStages, setAnalysisStages] = useState<
    Record<number, AnalysisStage>
  >({});

  useEffect(() => {
    document.documentElement.classList.toggle("dark", theme === "dark");
    localStorage.setItem(THEME_STORAGE_KEY, theme);
  }, [theme]);

  // One subscription for the whole app; cards read from `analysisStages` by
  // game_id, so their indicator doesn't reset every time the card unmounts.
  useAppEvent("analysis-status", ({ game_id, stage }) =>
    setAnalysisStages((prev) => ({ ...prev, [game_id]: stage })),
  );

  const toggleTheme = () =>
    setTheme((prev) => (prev === "dark" ? "light" : "dark"));

  return (
    <AppContext.Provider
      value={{ theme, setTheme, toggleTheme, analysisStages }}
    >
      {children}
    </AppContext.Provider>
  );
}
