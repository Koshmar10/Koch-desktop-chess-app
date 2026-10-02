import { Eye, EyeOff } from "lucide-react";
import type { EyeState } from "./rowControls";

const STATE_CLASS: Record<EyeState, string> = {
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
 */
const EyeIndicator = ({ state }: { state: EyeState }) => (
  <span
    aria-hidden
    className={`flex shrink-0 p-1 transition-colors duration-150 ${STATE_CLASS[state]}`}
  >
    {state === "hidden" ? <EyeOff size={15} /> : <Eye size={15} />}
  </span>
);

export default EyeIndicator;
