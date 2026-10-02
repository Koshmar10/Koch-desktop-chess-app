import type { MaskSelection } from "../types";

/**
 * Where a row's finding stands on the board: not drawn, drawn because the
 * pointer is on the row, or switched on by a click and staying put.
 */
export type EyeState = "hidden" | "previewing" | "shown";

/** Everything a row needs to act as the control for one finding. */
export interface RowControls {
  state: EyeState;
  // What gets drawn, for the accessible name — e.g. "this pin".
  label: string;
  onToggle: () => void;
  onPreview: (isPreviewing: boolean) => void;
}

// A faint wash under the pointer, so a row reads as something you can
// point at before you've learned that it does anything.
export const HOVER_ROW_CLASS = "-mx-1 rounded-md px-1 hover:bg-foreground/5";

const eyeState = (isShown: boolean, isPreviewing: boolean): EyeState => {
  if (isShown) return "shown";
  return isPreviewing ? "previewing" : "hidden";
};

/**
 * Controls for a row that draws one overlay mask — a pin, a king and its
 * attackers, square control, the best move. Every row is the same control
 * over a different id, whatever it draws.
 */
export const maskControls = (
  id: string,
  label: string,
  { shown, previewId, onToggle, onPreview }: MaskSelection,
): RowControls => ({
  state: eyeState(shown.has(id), previewId === id),
  label,
  onToggle: () => onToggle(id),
  onPreview: (isPreviewing) => onPreview(isPreviewing ? id : null),
});
