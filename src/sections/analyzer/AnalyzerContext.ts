import { createContext, Dispatch, SetStateAction, useContext } from "react";
import { AnalysisStage } from "../../api/bindings/AnalysisStage";
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
  // Where the loaded game's analysis is, if it's been queued this session.
  analysisStage: AnalysisStage | undefined,
  // Queues the loaded game for a fresh analysis pass. Resolves once it's
  // queued; the replay reloads by itself when the pass is done.
  reanalyze: () => Promise<void>,
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
  // Goes up by one whenever the sandbox is replaced or rewound rather than
  // moved forward — a fresh start, an import, an undo — so anything keyed
  // to its old positions (the live engine's search) knows to start over.
  generation: number,
  // Each of these replaces the whole sandbox and puts the view on its last
  // position. `start` begins on `fen`, the standard start when null; `load`
  // on a pasted FEN or PGN, rejecting with a message to show if it's
  // neither; `undo` takes back the last move.
  start: (fen: string | null) => Promise<void>,
  load: (notation: string) => Promise<void>,
  undo: () => Promise<void>,
}

export interface AnalyzerContextValue {
  loadedGameId: number | null,
  // The only way to switch games: it resets the game's view along with the
  // id. A bare id setter would let a game open on the last one's ply.
  openGame: (gameId: number) => void,
  game: GameAnalyzerContextValue
  sandbox: SandboxAnalyzerContextValue
  isCopyOverlayOpen: boolean,
  setIsCopyOverlayOpen: Dispatch<SetStateAction<boolean>>
  isGameSelectOverlayOpen: boolean,
  setIsGameSelectOverlayOpen: Dispatch<SetStateAction<boolean>>
  isImportOverlayOpen: boolean,
  setIsImportOverlayOpen: Dispatch<SetStateAction<boolean>>
}
  
export const AnalyzerContext = createContext<AnalyzerContextValue | null>(null);

export function useAnalyzerContext(): AnalyzerContextValue {
  const ctx = useContext(AnalyzerContext);
  if (!ctx) {
    throw new Error("useAnalyzerContext must be used inside <AnalyzerProvider>");
  }
  return ctx;
}
