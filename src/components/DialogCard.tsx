import type { ReactNode } from "react";
import { X } from "lucide-react";

const CLOSE_ICON_SIZE = 18;

// The two widths a dialog comes in, so cards opened side by side in the
// app line up instead of each picking its own number.
export const NARROW_DIALOG_PX = 360;
export const WIDE_DIALOG_PX = 480;

interface DialogCardProps {
  // Left out for a card whose content is its own heading — the result
  // card's verdict.
  title?: string;
  // Given, an X in the corner closes the card. Left out, the card closes
  // some other way: a toolbar toggle, or picking something inside it.
  onClose?: () => void;
  widthPx: number;
  // Only for a card whose content scrolls inside it, like a list. Every
  // other card grows to fit what's in it.
  heightPx?: number;
  children: ReactNode;
}

/**
 * The card every dialog is drawn on — over the whole screen in a `Popup`,
 * or over the board in a `BoardOverlay`. Seven dialogs used to draw their
 * own, in four different looks: different borders, corners and shadows,
 * and a title that was an h1 in some and an h2 in others.
 */
const DialogCard = ({
  title,
  onClose,
  widthPx,
  heightPx,
  children,
}: DialogCardProps) => (
  <div
    // Never wider than the window, whatever width was asked for.
    className="relative flex max-w-[92vw] flex-col gap-4 rounded-lg border border-border bg-card p-6 shadow-xl"
    style={{ width: widthPx, height: heightPx }}
  >
    {onClose && (
      <button
        type="button"
        aria-label="Close"
        onClick={onClose}
        className="absolute top-3 right-3 cursor-pointer rounded-md p-1 text-foreground/40 transition-colors hover:bg-primary/10 hover:text-foreground/80"
      >
        <X size={CLOSE_ICON_SIZE} />
      </button>
    )}
    {title && (
      <h2 className="text-lg font-medium text-foreground/90">{title}</h2>
    )}
    {children}
  </div>
);

export default DialogCard;
