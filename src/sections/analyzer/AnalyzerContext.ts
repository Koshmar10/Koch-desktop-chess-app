import { createContext, Dispatch, SetStateAction, useContext } from "react";
import { GameReplay } from "../../api/bindings/GameReplay";
import { PieceColor } from "../../api/bindings/PieceColor";
import { ReplayPosition } from "../../api/bindings/ReplayPosition";
import { SandboxStarted } from "../../api/bindings/SandboxStarted";
import { Square } from "../../api/bindings/Square";
import { PanelTab } from "./panel/SidePanel";

// Always present, with its setters, so a game can be loaded into it; what
// starts empty is the replay — null until the first game is opened.
export interface GameAnalyzerContextValue {
  replay: GameReplay | null,
  error: string | null,
  isLoading: boolean,
  viewedPly: number,
  setViewedPly: Dispatch<SetStateAction<number>>,
  isFlipped: boolean,
  setIsFlipped: Dispatch<SetStateAction<boolean>>,
  shownMasks: ReadonlySet<string>,
  setShownMasks: Dispatch<SetStateAction<ReadonlySet<string>>>,
  selectedTab?: PanelTab,
  setSelectedTab: Dispatch<SetStateAction<PanelTab>>,
}
export interface SandboxAnalyzerContextValue {
  // Same shape a game replay's positions have — `positions[0]` the start —
  // so the analyzer renders both modes from one list. Empty only until the
  // backend's sandbox has started.
  positions: ReplayPosition[],
  // The sandbox's own, so stepping through it never moves the game's.
  viewedPly: number,
  setViewedPly: Dispatch<SetStateAction<number>>,
  // Its own overlays too — what's lit in the sandbox stays put when you
  // switch to a game and back, and never lights up over the game.
  shownMasks: ReadonlySet<string>,
  setShownMasks: Dispatch<SetStateAction<ReadonlySet<string>>>,
  // Whose turn it is in the latest position, and what may move there —
  // what the board needs to let you drag pieces.
  turn: PieceColor,
  legalMoves: SandboxStarted["legal_moves"],
  // Plays a move from the latest position and moves the view onto it.
  // Resolves to the new position; rejects if the move is illegal.
  makeMove: (from: Square, to: Square) => Promise<ReplayPosition>,
}

export interface AnalyzerContextValue {
  loadedGameId: number | null,
  setLoadedGameId: Dispatch<SetStateAction<number | null>>
  game: GameAnalyzerContextValue
  sandbox: SandboxAnalyzerContextValue
}

export const AnalyzerContext = createContext<AnalyzerContextValue | null>(null);

export function useAnalyzerContext(): AnalyzerContextValue {
  const ctx = useContext(AnalyzerContext);
  if (!ctx) {
    throw new Error("useAnalyzerContext must be used inside <AnalyzerProvider>");
  }
  return ctx;
}
