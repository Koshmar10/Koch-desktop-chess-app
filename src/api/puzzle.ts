import { invoke } from "@tauri-apps/api/core";
import type { ChessPuzzle } from "./bindings/ChessPuzzle";

export const getPuzzle = async (): Promise<ChessPuzzle> => {
  return invoke<ChessPuzzle>("get_puzzle", {})
    .then((response) => {
      return response;
    })
    .catch((err) => {
      console.error("get_puzzle failed:", err);
      throw err;
    });
};
