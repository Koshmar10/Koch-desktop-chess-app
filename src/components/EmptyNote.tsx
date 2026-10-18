import type { ReactNode } from "react";

interface EmptyNoteProps {
  children: ReactNode;
}

/**
 * What a list or section says when it has nothing to show — "No pins",
 * "No moves yet", "Not enough moves to graph". One quiet look for every
 * one of them, so an empty section reads as empty rather than as broken,
 * whichever screen it's on.
 */
const EmptyNote = ({ children }: EmptyNoteProps) => (
  <p className="text-xs text-foreground/40 italic">{children}</p>
);

export default EmptyNote;
