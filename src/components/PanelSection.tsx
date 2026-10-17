import type { ReactNode } from "react";
import type { LucideIcon } from "lucide-react";

const TITLE_ICON_SIZE = 16;

interface PanelSectionProps {
  title: string;
  // Drawn before the title, in the accent colour — for sections that
  // stand alone, like the post-game stat pages, rather than sit in a tab.
  icon?: LucideIcon;
  children: ReactNode;
  /**
   * A control at the right end of the title row, for an action on the
   * section as a whole — the Game section's "Change" — rather than on any
   * one row inside it.
   */
  action?: ReactNode;
  /**
   * Take the column's leftover height and scroll internally, instead of
   * sizing to content. At most one section per tab should fill — two
   * competing for the slack is how a panel ends up with two scrollbars.
   */
  fill?: boolean;
  /**
   * Rule beneath the section, for one that sits above content it should
   * read as separate from rather than merely above.
   */
  divided?: boolean;
}

/**
 * One titled block of a side panel — the analyzer's tabs, the play
 * screen's panel, a post-game stat page.
 *
 * Each of those was growing its own copy of the same `<section>` plus
 * uppercase `<h3>`, which had already drifted on spacing. Keeping it in
 * one place is what makes every panel in the app read as the same kind of
 * thing, rather than each screen having its own.
 */
const PanelSection = ({
  title,
  icon: Icon,
  children,
  action,
  fill = false,
  divided = false,
}: PanelSectionProps) => {
  const sizing = fill ? "min-h-0 flex-1" : "shrink-0";
  const rule = divided ? "border-b-[1px] border-border/60 pb-3" : "";

  return (
    <section className={`flex flex-col gap-2 ${sizing} ${rule}`}>
      <div className="flex shrink-0 items-center justify-between gap-2">
        <h3 className="flex items-center gap-2 text-sm font-semibold tracking-wide text-foreground/60 uppercase">
          {Icon && <Icon size={TITLE_ICON_SIZE} className="text-primary" />}
          {title}
        </h3>
        {action}
      </div>
      {fill ? (
        <div className="min-h-0 flex-1 overflow-y-auto">{children}</div>
      ) : (
        children
      )}
    </section>
  );
};

export default PanelSection;
