import type { ReactNode } from "react";

/** What every text button takes, whichever kind it is. */
interface ButtonProps {
  // The label, with an icon before it if the button has one.
  children: ReactNode;
  onClick?: () => void;
  disabled?: boolean;
  title?: string;
}

const BASE_CLASS =
  "flex shrink-0 cursor-pointer items-center justify-center gap-2 rounded-md px-3 py-1.5 text-sm transition-colors disabled:pointer-events-none disabled:opacity-40";

/**
 * The shared text button, which each kind below fills in with its look —
 * the same base-and-wrappers shape as `InteractiveRow` and its rows. It
 * owns size, shape and the disabled state, so the kinds differ only in
 * colour; dialogs and the settings page each used to spell all of this
 * out per button.
 */
const Button = ({
  look,
  children,
  onClick,
  disabled,
  title,
}: ButtonProps & { look: string }) => (
  <button
    type="button"
    onClick={onClick}
    disabled={disabled}
    title={title}
    className={`${BASE_CLASS} ${look}`}
  >
    {children}
  </button>
);

const PRIMARY_LOOK =
  "bg-primary font-medium text-primary-foreground hover:bg-primary/90";
const OUTLINE_LOOK =
  "border border-border text-foreground/90 hover:bg-border/60";
const GHOST_LOOK = "text-foreground/70 hover:bg-border/60";

/** The action a dialog or section exists for — Save, Import, Copy. */
export const PrimaryButton = (props: ButtonProps) => (
  <Button {...props} look={PRIMARY_LOOK} />
);

/** An action that stands on its own beside others, none of them primary. */
export const OutlineButton = (props: ButtonProps) => (
  <Button {...props} look={OUTLINE_LOOK} />
);

/** The way out next to a primary action — Cancel, Done. */
export const GhostButton = (props: ButtonProps) => (
  <Button {...props} look={GHOST_LOOK} />
);
