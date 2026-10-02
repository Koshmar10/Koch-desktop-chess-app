import type { ReactNode } from "react";
import type { RowControls } from "./rowControls";

const TOGGLE_KEYS = new Set(["Enter", " "]);

/**
 * A row that is the control for its finding: rest on it to preview it on
 * the board, click to switch it on and leave it there, click again to
 * switch it off. Clicking an icon on every row to see anything made the
 * panel slow to explore; this keeps a look free and a commitment one
 * click away.
 *
 * Works from the keyboard too — focusing a row previews it, and Enter or
 * Space toggles it — since the row is now the only control there is.
 */
const InteractiveRow = ({
  controls,
  className = "",
  children,
}: {
  controls: RowControls;
  className?: string;
  children: ReactNode;
}) => (
  <div
    role="button"
    tabIndex={0}
    aria-pressed={controls.state === "shown"}
    aria-label={`${controls.state === "shown" ? "Hide" : "Show"} ${controls.label} on the board`}
    onClick={controls.onToggle}
    onKeyDown={(event) => {
      if (!TOGGLE_KEYS.has(event.key)) return;
      event.preventDefault();
      controls.onToggle();
    }}
    onMouseEnter={() => controls.onPreview(true)}
    onMouseLeave={() => controls.onPreview(false)}
    onFocus={() => controls.onPreview(true)}
    onBlur={() => controls.onPreview(false)}
    className={`cursor-pointer outline-none transition-colors duration-100 focus-visible:ring-2 focus-visible:ring-primary/50 ${className}`}
  >
    {children}
  </div>
);

export default InteractiveRow;
