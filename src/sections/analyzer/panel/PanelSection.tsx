import type { ReactNode } from "react";

interface PanelSectionProps {
  title: string;
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
 * One titled block inside a side-panel tab.
 *
 * All three tabs were growing their own copy of the same `<section>` plus
 * uppercase `<h3>`, which had already drifted on spacing. Keeping it in
 * one place is what makes switching tabs feel like moving within a panel
 * rather than between three of them.
 */
const PanelSection = ({
  title,
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
        <h3 className="text-sm font-semibold tracking-wide text-foreground/60 uppercase">
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
