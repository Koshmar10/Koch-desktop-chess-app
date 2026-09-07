import { useEffect, useState, type ReactNode } from "react";
import { AppContext, type Theme } from "./AppContext";

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

// App-wide state that outlives any one route or game — theme for now, the
// place to hang future app preferences (board style, sound, etc.).
// GameProvider stays nested inside this, not the other way round: a game
// is one thing the app contains, the app isn't one thing a game contains.
export function AppProvider({ children }: AppProviderProps) {
  const [theme, setTheme] = useState<Theme>(initialTheme);

  useEffect(() => {
    document.documentElement.classList.toggle("dark", theme === "dark");
    localStorage.setItem(THEME_STORAGE_KEY, theme);
  }, [theme]);

  const toggleTheme = () =>
    setTheme((prev) => (prev === "dark" ? "light" : "dark"));

  return (
    <AppContext.Provider value={{ theme, setTheme, toggleTheme }}>
      {children}
    </AppContext.Provider>
  );
}
