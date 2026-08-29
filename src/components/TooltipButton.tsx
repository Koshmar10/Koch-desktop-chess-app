import type { ReactNode } from "react";
import { Tooltip } from "./Tooltip";

const buttonCls =
  "relative text-foreground/90 transition-all duration-150 ease-out p-2 rounded-md border-[1px] border-primary/20 bg-card hover:bg-primary/50 cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed disabled:hover:bg-card";

interface TooltipButtonProps {
  icon: ReactNode;
  tooltip: string | null;
  onClick?: () => void;
  disabled?: boolean;
}

export const TooltipButton = ({
  icon,
  tooltip,
  onClick,
  disabled,
}: TooltipButtonProps) => {
  return (
    <Tooltip label={tooltip} color="primary">
      <button
        type="button"
        className={buttonCls}
        onClick={onClick}
        disabled={disabled}
      >
        {icon}
      </button>
    </Tooltip>
  );
};
