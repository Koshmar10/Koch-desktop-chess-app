import { JSX, useState } from "react";

interface IconButtonProps {
  icon: JSX.Element;
  tooltip: string | null;
  disabled?: boolean;
  active?: boolean;
  onClick?: () => void;
  /** Where the tooltip appears relative to the button */
  tooltipPosition?: "top" | "bottom";
  /** Visual variant */
  variant?: "default" | "rounded";
}

export function IconButton({
  icon,
  tooltip,
  disabled = false,
  active = false,
  onClick,
  tooltipPosition = "top",
  variant = "default",
}: IconButtonProps) {
  const [tooltipOpen, setTooltipOpen] = useState<boolean>(false);

  const baseBtnCls = [
    "relative text-foreground/90 transition-all duration-150 ease-out",
    variant === "rounded"
      ? "rounded-full p-2 border border-transparent"
      : "p-2 rounded-md border-[1px] border-primary/20",
    variant === "rounded"
      ? active && !disabled
        ? "bg-primary/70"
        : "bg-card"
      : "bg-card",
    disabled
      ? "opacity-40 grayscale pointer-events-none cursor-not-allowed"
      : variant === "rounded"
        ? "hover:bg-primary/50 cursor-pointer"
        : "hover:scale-105",
  ]
    .filter(Boolean)
    .join(" ");

  const tooltipBg = disabled
    ? "bg-primary/30 text-foreground/70"
    : "bg-primary text-foreground";

  const tooltipStateCls =
    tooltipOpen && !disabled
      ? "opacity-100 scale-100 pointer-events-auto"
      : "opacity-0 scale-95 pointer-events-none";

  const tooltipPositionCls =
    tooltipPosition === "bottom"
      ? "top-full mt-2"
      : "-top-10";

  const arrowCls =
    tooltipPosition === "bottom"
      ? "top-0 -translate-y-1"
      : "bottom-0 translate-y-1";

  return (
    <div className="relative flex flex-col items-center" aria-disabled={disabled}>
      <button
        type="button"
        className={baseBtnCls}
        onMouseEnter={() => !disabled && setTooltipOpen(true)}
        onMouseLeave={() => !disabled && setTooltipOpen(false)}
        onClick={!disabled && onClick ? onClick : undefined}
        disabled={disabled}
        tabIndex={disabled ? -1 : 0}
      >
        {icon}
      </button>
      {tooltip && (
        <div
          className={`absolute z-20 ${tooltipPositionCls} left-1/2 -translate-x-1/2 text-sm px-2 py-[2px] rounded shadow w-fit flex items-center transition-all duration-200 ease-out ${tooltipBg} ${tooltipStateCls}`}
        >
          <span className="whitespace-nowrap">{tooltip}</span>
          <div
            className={`absolute left-1/2 ${arrowCls} -translate-x-1/2 w-2 h-2 rotate-45 z-10 ${disabled ? "bg-primary/30" : "bg-primary"}`}
          />
        </div>
      )}
    </div>
  );
}
