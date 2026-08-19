import type { ReactNode } from "react";

const buttonCls =
  "relative text-foreground/90 transition-all duration-150 ease-out p-2 rounded-md border-[1px] border-primary/20 bg-card hover:bg-primary/50 cursor-pointer";

const tooltipPositionCls = "absolute z-20 -top-10 left-1/2 -translate-x-1/2";
const tooltipShapeCls = "text-sm px-2 py-[2px] rounded shadow w-fit flex items-center";
const tooltipColorCls = "bg-primary text-foreground";
const tooltipRevealCls =
  "transition-all duration-200 ease-out opacity-0 scale-95 pointer-events-none group-hover:opacity-100 group-hover:scale-100 group-hover:pointer-events-auto";
const tooltipCls = `${tooltipPositionCls} ${tooltipShapeCls} ${tooltipColorCls} ${tooltipRevealCls}`;

const arrowPositionCls = "absolute left-1/2 bottom-0 translate-y-1 -translate-x-1/2";
const arrowShapeCls = "w-2 h-2 rotate-45 z-10 bg-primary";
const arrowCls = `${arrowPositionCls} ${arrowShapeCls}`;

interface TooltipButtonProps {
  icon: ReactNode;
  tooltip: string | null;
}

export function TooltipButton({ icon, tooltip }: TooltipButtonProps) {
  return (
    <div className="group relative flex flex-col items-center">
      <button type="button" className={buttonCls}>
        {icon}
      </button>
      {tooltip && (
        <div className={tooltipCls}>
          <span className="whitespace-nowrap">{tooltip}</span>
          <div className={arrowCls} />
        </div>
      )}
    </div>
  );
}
