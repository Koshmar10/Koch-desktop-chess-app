import { NOTATION_TEXTAREA_CLASS } from "../../../components/controlStyles";

interface NotationFieldProps {
  id: string;
  label: string;
  value: string;
  rows: number;
  // Given, the field takes typing and pasting. Left out, it's for copying
  // out: read-only, and selected whole on focus, so one click and a copy
  // takes the lot.
  onChange?: (value: string) => void;
  autoFocus?: boolean;
}

/** A labelled box of FEN or PGN, for the analyzer's copy and import cards. */
const NotationField = ({
  id,
  label,
  value,
  rows,
  onChange,
  autoFocus,
}: NotationFieldProps) => (
  <div className="flex flex-col gap-1">
    <label htmlFor={id} className="ml-1 text-sm text-card-foreground/80">
      {label}
    </label>
    <textarea
      id={id}
      rows={rows}
      value={value}
      spellCheck={false}
      autoFocus={autoFocus}
      className={NOTATION_TEXTAREA_CLASS}
      readOnly={!onChange}
      onChange={onChange && ((e) => onChange(e.target.value))}
      onFocus={onChange ? undefined : (e) => e.currentTarget.select()}
    />
  </div>
);

export default NotationField;
