import type { ReactNode } from "react";

interface PanelPlaceholderProps {
  title: string;
  icon: ReactNode;
  // Which ticket fills this column in. Named rather than vague ("coming
  // soon") so an empty panel is traceable to the work that's missing,
  // instead of reading as something that broke.
  blockedOn: string;
  widthPx: number;
  // Which edge divides this column from the board area next to it.
  borderSide: "left" | "right";
}

const PanelPlaceholder = ({
  title,
  icon,
  blockedOn,
  widthPx,
  borderSide,
}: PanelPlaceholderProps) => (
  <div
    className={`flex h-full shrink-0 flex-col items-center gap-3 border-border/80 bg-secondary/20 p-4 ${borderSide === "left" ? "border-l-[1px]" : "border-r-[1px]"}`}
    style={{ width: widthPx }}
  >
    <div className="text-foreground/30">{icon}</div>
    <h3 className="text-center text-xs font-semibold tracking-wide text-foreground/40 uppercase">
      {title}
    </h3>
    <span className="text-center text-xs text-foreground/25">{blockedOn}</span>
  </div>
);

export default PanelPlaceholder;
