import type { ReactNode } from "react";
import { GhostButton, PrimaryButton } from "../../components/buttons";

// Two columns: label + `* hint` stacked on the left, the control on a
// fixed right rail so every control shares the same right edge and the
// number fields line up with each other.
export const SettingRow = ({
  label,
  hint,
  children,
}: {
  label: string;
  hint?: string;
  children: ReactNode;
}) => (
  <div className="flex items-center justify-between gap-6 py-3">
    <div className="flex min-w-0 flex-col gap-0.5">
      <span className="text-foreground/80">{label}</span>
      {hint && (
        <p className="text-xs leading-snug text-foreground/45">* {hint}</p>
      )}
    </div>
    <div className="flex w-56 shrink-0 justify-end">{children}</div>
  </div>
);

// A titled group of related settings — separated by heading and spacing
// only, no card or fill. The subtitle carries the section's scope so
// individual row hints can stay short.
export const Section = ({
  title,
  subtitle,
  dirty,
  onSave,
  onCancel,
  children,
}: {
  title: string;
  subtitle: string;
  dirty: boolean;
  onSave: () => void;
  onCancel: () => void;
  children: ReactNode;
}) => (
  <section>
    <h2 className="text-xl text-foreground/90">{title}</h2>
    <p className="mt-0.5 text-sm text-foreground/50">{subtitle}</p>
    <div className="mt-2 divide-y divide-border/40">{children}</div>
    <div className="mt-6 flex justify-end gap-2">
      <GhostButton onClick={onCancel} disabled={!dirty}>
        Cancel
      </GhostButton>
      <PrimaryButton onClick={onSave} disabled={!dirty}>
        Save
      </PrimaryButton>
    </div>
  </section>
);
