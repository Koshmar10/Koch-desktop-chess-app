import { invoke } from "@tauri-apps/api/core";
import type { ChessPuzzle } from "./bindings/ChessPuzzle";

export const getPuzzle = (): Promise<ChessPuzzle> =>
  invoke<ChessPuzzle>("get_puzzle");
