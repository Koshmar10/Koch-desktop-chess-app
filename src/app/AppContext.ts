import { createContext, useContext } from "react";
import type { AnalysisStage } from "../api/bindings/AnalysisStage";

export type Theme = "light" | "dark";

export interface AppContextValue {
  theme: Theme;
  setTheme: (theme: Theme) => void;
  toggleTheme: () => void;
  /**
   * Latest analysis stage per game_id, accumulated across the whole
   * session from the backend `analysis-status` stream. Lives here so a
   * history card's queued/running indicator survives the card unmounting
   * (route change, scroll, list re-key).
   */
  analysisStages: Record<number, AnalysisStage>;
}

export const AppContext = createContext<AppContextValue | null>(null);

export const useAppContext = (): AppContextValue => {
  const ctx = useContext(AppContext);
  if (!ctx) {
    throw new Error("useAppContext must be used inside <AppProvider>");
  }
  return ctx;
};

export const useAnalysisStage = (gameId: number): AnalysisStage | undefined =>
  useAppContext().analysisStages[gameId];
