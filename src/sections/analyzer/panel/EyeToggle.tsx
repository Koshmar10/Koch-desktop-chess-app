import { Eye, EyeOff } from "lucide-react";

interface EyeToggleProps {
  shown: boolean;
  onToggle: () => void;
  /** What gets drawn, for the tooltip — e.g. "this pin". */
  label: string;
}

/**
 * Whether one finding is drawn on the board.
 *
 * The same open/closed eye a password field uses, for the same reason:
 * the icon states what you'd get by clicking, and the pair reads as
 * on/off without a label. A panel row is otherwise indistinguishable
 * from a row that happens to be lit.
 */
const EyeToggle = ({ shown, onToggle, label }: EyeToggleProps) => (
  <button
    type="button"
    onClick={onToggle}
    aria-pressed={shown}
    title={shown ? `Hide ${label} on the board` : `Show ${label} on the board`}
    className={`flex shrink-0 cursor-pointer items-center rounded p-1 transition-colors duration-150 ${
      shown
        ? "text-primary"
        : "text-foreground/25 hover:bg-primary/10 hover:text-foreground/60"
    }`}
  >
    {shown ? <Eye size={15} /> : <EyeOff size={15} />}
  </button>
);

export default EyeToggle;
