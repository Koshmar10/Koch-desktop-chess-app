import { createContext, useContext } from "react";

/** What the chat needs from the board it sits beside. */
export interface ChatBoard {
  // The ply on the board, as an index (-1 is the start position).
  viewedPly: number;
  onGoToPly: (ply: number) => void;
  // The move the pointer is on in an answer, as UCI; null once it leaves.
  onPreviewMove: (uci: string | null) => void;
}

/**
 * Context rather than props so a rendered answer doesn't depend on the ply:
 * stepping through the game then re-renders only the chips and headers that
 * read it, not every answer's markdown.
 */
export const ChatBoardContext = createContext<ChatBoard | null>(null);

export const useChatBoard = (): ChatBoard => {
  const board = useContext(ChatBoardContext);
  if (board === null) {
    throw new Error(
      "useChatBoard must be used inside a ChatBoardContext provider",
    );
  }
  return board;
};
