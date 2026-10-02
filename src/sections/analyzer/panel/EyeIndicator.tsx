import { Eye, EyeOff } from "lucide-react";
import type { MaskVisibility } from "./maskSelection";

const VISIBILITY_CLASS: Record<MaskVisibility, string> = {
  hidden: "text-foreground/25",
  previewing: "text-foreground/60",
  shown: "text-primary",
};

/**
 * Where a row's finding stands on the board, at a glance: closed and dim
 * when it isn't drawn, open but dim while you're only hovering it, open
 * and in the accent colour once a click has switched it on — the
 * difference between "I'm looking at this" and "I left this on".
 *
 * Not a button. The whole row is the control, so this only reports; a
 * second click target inside it would mean two ways to do one thing, and
 * a button nested inside a button.
 *
 * Pushes itself to the far end of the row it sits in.
 */
const EyeIndicator = ({ visibility }: { visibility: MaskVisibility }) => (
  <span
    aria-hidden
    className={`ml-auto flex shrink-0 p-1 transition-colors duration-150 ${VISIBILITY_CLASS[visibility]}`}
  >
    {visibility === "hidden" ? <EyeOff size={15} /> : <Eye size={15} />}
  </span>
);

export default EyeIndicator;
