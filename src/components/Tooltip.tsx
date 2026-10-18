import type { ReactNode } from "react";

const TOOLTIP_POSITION_CLASS =
  "absolute z-20 -top-10 left-1/2 -translate-x-1/2";
const TOOLTIP_SHAPE_CLASS =
  "text-sm px-2 py-[2px] rounded shadow w-fit flex items-center";
const TOOLTIP_TEXT_CLASS = "text-foreground";
const TOOLTIP_REVEAL_CLASS =
  "transition-all duration-200 ease-out opacity-0 scale-95 pointer-events-none group-hover:opacity-100 group-hover:scale-100 group-hover:pointer-events-auto";

const ARROW_POSITION_CLASS =
  "absolute left-1/2 bottom-0 translate-y-1 -translate-x-1/2";
const ARROW_SHAPE_CLASS = "w-2 h-2 rotate-45 z-10";

type TooltipColor = "primary" | "secondary";

// Written out whole rather than built as `bg-${color}`: Tailwind only
// generates classes it finds spelled out in the source, so a built one
// works only by luck — while some other file happens to use it.
const TOOLTIP_BG_CLASS: Record<TooltipColor, string> = {
  primary: "bg-primary",
  secondary: "bg-secondary",
};

interface TooltipProps {
  color?: TooltipColor;
  label: string | null;
  children: ReactNode;
}

const Tooltip = ({ color = "secondary", label, children }: TooltipProps) => {
  const bgClass = TOOLTIP_BG_CLASS[color];
  const tooltipClass = `${TOOLTIP_POSITION_CLASS} ${TOOLTIP_SHAPE_CLASS} ${bgClass} ${TOOLTIP_TEXT_CLASS} ${TOOLTIP_REVEAL_CLASS}`;
  const arrowClass = `${ARROW_POSITION_CLASS} ${ARROW_SHAPE_CLASS} ${bgClass}`;

  return (
    <div className="group relative flex flex-col items-center w-fit h-fit">
      {children}
      {label && (
        <div className={tooltipClass}>
          <span className="whitespace-nowrap">{label}</span>
          <div className={arrowClass} />
        </div>
      )}
    </div>
  );
};

export default Tooltip;
