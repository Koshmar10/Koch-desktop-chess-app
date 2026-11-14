import type { ReactNode } from "react";
import type { LucideIcon } from "lucide-react";

const ICON_SIZE_PX = 40;

interface EmptyStateProps {
  icon: LucideIcon;
  // One sentence: why there's nothing here, and what would put something
  // here.
  message: string;
  // The buttons that fill it.
  children?: ReactNode;
}

/**
 * What a whole screen shows when it has nothing to show, so an empty
 * screen reads as empty, not broken, and says how to fill it. `EmptyNote`
 * is the quiet version for a section.
 */
const EmptyState = ({ icon: Icon, message, children }: EmptyStateProps) => (
  <div className="flex flex-col items-center gap-4 px-6 py-10 text-center">
    <Icon size={ICON_SIZE_PX} className="text-foreground/30" />
    <p className="max-w-sm text-sm text-foreground/60">{message}</p>
    {children && (
      <div className="flex flex-wrap justify-center gap-2">{children}</div>
    )}
  </div>
);

export default EmptyState;
