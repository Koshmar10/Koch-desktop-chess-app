import type { ReactNode } from "react";
import { Tooltip } from "./Tooltip";

const buttonCls =
  "relative text-foreground/90 transition-all duration-150 ease-out p-2 rounded-md border-[1px] border-primary/20 bg-card hover:bg-primary/50 cursor-pointer";

interface TooltipButtonProps {
  icon: ReactNode;
  tooltip: string | null;
}

export function TooltipButton({ icon, tooltip }: TooltipButtonProps) {
  return (
    <Tooltip label={tooltip} color="primary/80">
      <button type="button" className={buttonCls}>
        {icon}
      </button>
    </Tooltip>
  );
}
