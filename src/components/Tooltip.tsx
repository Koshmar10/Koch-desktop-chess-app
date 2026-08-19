import type { ReactNode } from "react";

const tooltipPositionCls = "absolute z-20 -top-10 left-1/2 -translate-x-1/2";
const tooltipShapeCls = "text-sm px-2 py-[2px] rounded shadow w-fit flex items-center";
const tooltipTextCls = "text-foreground";
const tooltipRevealCls =
  "transition-all duration-200 ease-out opacity-0 scale-95 pointer-events-none group-hover:opacity-100 group-hover:scale-100 group-hover:pointer-events-auto";

const arrowPositionCls = "absolute left-1/2 bottom-0 translate-y-1 -translate-x-1/2";
const arrowShapeCls = "w-2 h-2 rotate-45 z-10";

interface TooltipProps {
  color?: string;
  label: string | null;
  children: ReactNode;
}

export function Tooltip({ color = "secondary", label, children }: TooltipProps) {
  const tooltipCls = `${tooltipPositionCls} ${tooltipShapeCls} bg-${color} ${tooltipTextCls} ${tooltipRevealCls}`;
  const arrowCls = `${arrowPositionCls} ${arrowShapeCls} bg-${color}`;

  return (
    <div className="group relative flex flex-col items-center w-fit h-fit">
      {children}
      {label && (
        <div className={tooltipCls}>
          <span className="whitespace-nowrap">{label}</span>
          <div className={arrowCls} />
        </div>
      )}
    </div>
  );
}
