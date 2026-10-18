import type { LucideIcon } from "lucide-react";
import Tooltip from "./Tooltip";

// One size for every icon button in the app, so a toolbar, the play
// controls and the history bar read as the same kind of control.
const ICON_SIZE_PX = 18;

const BUTTON_CLASS =
  "relative text-foreground/90 transition-all duration-150 ease-out p-2 rounded-md border-[1px] cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed";

// A toggle that's currently on keeps its filled look instead of only
// showing it on hover, so the state is readable without pointing at it.
const ACTIVE_CLASS =
  "border-primary bg-primary/60 disabled:hover:bg-primary/60";
const INACTIVE_CLASS =
  "border-primary/20 bg-card hover:bg-primary/50 disabled:hover:bg-card";

interface TooltipButtonProps {
  // The icon component itself, not an element: the button draws it, at
  // the one size every icon button shares.
  icon: LucideIcon;
  // For an icon that moves or turns to show state — a spinner, the
  // half-circle that rotates to show which colour you'll play.
  iconClassName?: string;
  tooltip: string | null;
  onClick?: () => void;
  disabled?: boolean;
  // For buttons that toggle something rather than fire and forget. Left
  // undefined, the button has no on/off state to show at all — which is
  // not the same as being off, so this is optional rather than defaulting
  // every button to `false`.
  active?: boolean;
}

const TooltipButton = ({
  icon: Icon,
  iconClassName,
  tooltip,
  onClick,
  disabled,
  active = false,
}: TooltipButtonProps) => {
  return (
    <Tooltip label={tooltip} color="primary">
      <button
        type="button"
        className={`${BUTTON_CLASS} ${active ? ACTIVE_CLASS : INACTIVE_CLASS}`}
        onClick={onClick}
        disabled={disabled}
      >
        <Icon size={ICON_SIZE_PX} className={iconClassName} />
      </button>
    </Tooltip>
  );
};

export default TooltipButton;
