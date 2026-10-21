import type { ReactNode } from "react";

interface PanelCardProps {
  children: ReactNode;
  /**
   * Take the column's leftover height instead of sizing to content, for a
   * card holding a section that scrolls (`PanelSection fill`). At most one
   * per column, for the same reason as `PanelSection`'s.
   */
  fill?: boolean;
}

/**
 * One card of a side panel. Play's cards and the analyzer's panel body
 * draw this same card with their own copies of the classes; it lives here
 * so a new panel doesn't add another.
 */
const PanelCard = ({ children, fill = false }: PanelCardProps) => {
  const sizing = fill ? "min-h-0 flex-1" : "shrink-0";

  return (
    <div
      className={`flex flex-col gap-4 rounded-lg border-[1px] border-border/80 bg-secondary/40 p-3 shadow-sm ${sizing}`}
    >
      {children}
    </div>
  );
};

export default PanelCard;
