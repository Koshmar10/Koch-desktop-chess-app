// Kept out of `controls.tsx` so that file stays component-only, which is
// what React Fast Refresh needs to hot-reload it.

export const CONTROL_CLASS =
  "px-2 py-1 text-sm rounded-md bg-input/30 border border-border outline-none focus:border-primary";

export const TEXT_INPUT_CLASS = `${CONTROL_CLASS} w-full`;

// For a box of FEN or PGN, pasted in or to copy out. The same border,
// fill and focus ring as `CONTROL_CLASS`, but monospaced and smaller, so
// moves and board rows line up — written out whole rather than built on
// `CONTROL_CLASS`, whose `text-sm` would fight a `text-xs` added after it.
export const NOTATION_TEXTAREA_CLASS =
  "w-full resize-none rounded-md border border-border bg-input/30 p-2 font-mono text-xs text-foreground outline-none focus:border-primary";
