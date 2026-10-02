import { createContext, useContext } from "react";
import type { MaskSelection } from "../types";

/**
 * Where a mask stands on the board: not drawn, drawn because the pointer
 * is on its row, or switched on by a click and staying put.
 */
export type MaskVisibility = "hidden" | "previewing" | "shown";

export const maskVisibility = (
  id: string,
  { shown, previewId }: MaskSelection,
): MaskVisibility => {
  if (shown.has(id)) return "shown";
  return previewId === id ? "previewing" : "hidden";
};

/**
 * The mask selection, for the rows that toggle masks. Provided by the
 * analyzer, which owns it because the board is the panel's sibling, and
 * read only by `InteractiveRow`.
 *
 * Context rather than a prop because every row in two tabs needs it:
 * passed down, it went through every component in between without being
 * used, and each row needed an adapter object to pick out its own slice.
 */
export const MaskSelectionContext = createContext<MaskSelection | null>(null);

/**
 * Throws outside the provider rather than handing back a selection that
 * does nothing — a row that silently ignores clicks is far harder to
 * trace than an error saying what's missing.
 */
export const useMaskSelection = (): MaskSelection => {
  const selection = useContext(MaskSelectionContext);
  if (selection === null) {
    throw new Error(
      "useMaskSelection must be used inside a MaskSelectionContext provider",
    );
  }
  return selection;
};
