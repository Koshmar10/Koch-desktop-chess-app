import type { ReactNode } from "react";
import { Tooltip } from "./Tooltip";

const buttonCls =
  "relative text-foreground/90 transition-all duration-150 ease-out p-2 rounded-md border-[1px] cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed";

// A toggle that's currently on keeps its filled look instead of only
// showing it on hover, so the state is readable without pointing at it.
const activeCls = "border-primary bg-primary/60 disabled:hover:bg-primary/60";
const inactiveCls =
  "border-primary/20 bg-card hover:bg-primary/50 disabled:hover:bg-card";

interface TooltipButtonProps {
  icon: ReactNode;
  tooltip: string | null;
  onClick?: () => void;
  disabled?: boolean;
  // For buttons that toggle something rather than fire and forget. Left
  // undefined, the button has no on/off state to show at all — which is
  // not the same as being off, so this is optional rather than defaulting
  // every button to `false`.
  active?: boolean;
}

export const TooltipButton = ({
  icon,
  tooltip,
  onClick,
  disabled,
  active = false,
}: TooltipButtonProps) => {
  return (
    <Tooltip label={tooltip} color="primary">
      <button
        type="button"
        className={`${buttonCls} ${active ? activeCls : inactiveCls}`}
        onClick={onClick}
        disabled={disabled}
      >
        {icon}
      </button>
    </Tooltip>
  );
};
