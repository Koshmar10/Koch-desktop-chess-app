import { useState } from "react"

// Shallow value-compare for the flat settings objects — good enough to
// drive "is this section dirty", since drafts are only ever built by
// spreading the saved object so key order stays stable.
export const isDirty = <T,>(a: T, b: T) => JSON.stringify(a) !== JSON.stringify(b)

// Editable copy of `saved` that resets whenever the saved reference
// changes (initial load, or a Save lifting new values to the parent).
// The render-phase compare is React's documented alternative to a
// state-resetting effect.
export const useDraft = <T,>(saved: T) => {
  const [draft, setDraft] = useState(saved)
  const [snapshot, setSnapshot] = useState(saved)
  if (saved !== snapshot) {
    setSnapshot(saved)
    setDraft(saved)
  }
  return [draft, setDraft] as const
}

export interface SectionState<T> {
  saved: T
  // Called on Save with the current draft. The parent treats this as
  // "these are the saved values now" (which disables the buttons);
  // persisting to the backend hangs off here once a save command exists.
  onSaved: (next: T) => void
}
