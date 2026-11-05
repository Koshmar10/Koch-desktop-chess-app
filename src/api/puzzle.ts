import { invoke } from "@tauri-apps/api/core";
import type { ChessPuzzle } from "./bindings/ChessPuzzle";
import type { NewPuzzleAttempt } from "./bindings/NewPuzzleAttempt";
import type { PuzzleStats } from "./bindings/PuzzleStats";

export const getPuzzle = (): Promise<ChessPuzzle> =>
  invoke<ChessPuzzle>("get_puzzle");

// Saves a solved puzzle, or one skipped after trying it.
export const recordPuzzleAttempt = (attempt: NewPuzzleAttempt): Promise<void> =>
  invoke<void>("record_puzzle_attempt", { attempt });

// Totals over every saved attempt, for the Home screen.
export const getPuzzleStats = (): Promise<PuzzleStats> =>
  invoke<PuzzleStats>("get_puzzle_stats");
