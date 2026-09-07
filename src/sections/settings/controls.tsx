import { useState } from "react"
import { Check, ChevronDown, ChevronUp, Eye, EyeOff } from "lucide-react"
import Dropdown from "../../components/Dropdown"
import { CONTROL_CLASS, TEXT_INPUT_CLASS } from "./constants"

// Masked by default with a reveal toggle — used for the OpenAI key, which
// is a secret. Note: nothing here persists it yet; per the repo's rules a
// key must go to the OS keychain via a backend command, never localStorage
// or a tracked file.
export const SecretInput = ({
  value,
  onChange,
  placeholder,
}: {
  value: string
  onChange: (value: string) => void
  placeholder?: string
}) => {
  const [revealed, setRevealed] = useState(false)
  return (
    <div className="flex w-full items-center gap-2">
      <input
        type={revealed ? "text" : "password"}
        value={value}
        onChange={(e) => onChange(e.target.value)}
        placeholder={placeholder}
        autoComplete="off"
        spellCheck={false}
        className={TEXT_INPUT_CLASS}
      />
      <button
        type="button"
        aria-label={revealed ? "Hide key" : "Show key"}
        onClick={() => setRevealed((r) => !r)}
        className="text-foreground/50 hover:text-foreground"
      >
        {revealed ? <EyeOff size={16} /> : <Eye size={16} />}
      </button>
    </div>
  )
}

export const NumberSetting = ({
  value,
  onChange,
  min,
  max,
  step = 1,
}: {
  value: number
  onChange: (value: number) => void
  min?: number
  max?: number
  step?: number
}) => {
  const clamped = (n: number) =>
    Math.min(max ?? Infinity, Math.max(min ?? -Infinity, n))
  return (
    <div
      className={`${CONTROL_CLASS} flex w-28 items-stretch overflow-hidden p-0`}
    >
      <input
        type="number"
        value={value}
        min={min}
        max={max}
        step={step}
        onChange={(e) => onChange(Number(e.target.value))}
        // Native number spinners render an opaque light box in the dark
        // webview — hidden here, the chevrons take their place.
        className="w-full bg-transparent px-2 py-1 outline-none [appearance:textfield] [&::-webkit-inner-spin-button]:hidden [&::-webkit-outer-spin-button]:hidden"
      />
      <div className="flex flex-col border-l border-border px-2 text-foreground/50">
        <button
          type="button"
          tabIndex={-1}
          aria-label="Increase"
          onClick={() => onChange(clamped(value + step))}
          className="flex flex-1 items-center hover:bg-border/60 hover:text-foreground"
        >
          <ChevronUp size={12} />
        </button>
        <button
          type="button"
          tabIndex={-1}
          aria-label="Decrease"
          onClick={() => onChange(clamped(value - step))}
          className="flex flex-1 items-center hover:bg-border/60 hover:text-foreground"
        >
          <ChevronDown size={12} />
        </button>
      </div>
    </div>
  )
}

export const ToggleSetting = ({
  checked,
  onChange,
}: {
  checked: boolean
  onChange: (checked: boolean) => void
}) => (
  <button
    type="button"
    role="switch"
    aria-checked={checked}
    onClick={() => onChange(!checked)}
    className={`flex w-10 shrink-0 rounded-full p-0.5 transition-colors ${checked ? "bg-primary" : "bg-border"
      }`}
  >
    <span
      className={`h-5 w-5 rounded-full bg-background transition-transform ${checked ? "translate-x-4" : ""
        }`}
    />
  </button>
)

export const SelectSetting = ({
  value,
  options,
  onChange,
}: {
  value: string
  options: readonly string[]
  onChange: (value: string) => void
}) => (
  <Dropdown
    trigger={({ toggle }) => (
      <button
        type="button"
        onClick={toggle}
        className={`${CONTROL_CLASS} flex w-36 items-center justify-between`}
      >
        {value}
        <ChevronDown size={14} />
      </button>
    )}
  >
    {(close) => (
      <div className="w-36 rounded-md border border-border bg-card py-1 shadow-lg">
        {options.map((option) => (
          <button
            key={option}
            type="button"
            onClick={() => {
              onChange(option)
              close()
            }}
            className="flex w-full items-center justify-between px-3 py-1.5 text-left text-sm hover:bg-border/60"
          >
            {option}
            {option === value && <Check size={14} />}
          </button>
        ))}
      </div>
    )}
  </Dropdown>
)
