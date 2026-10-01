// Kept out of `controls.tsx` so that file stays component-only, which is
// what React Fast Refresh needs to hot-reload it.

export const CONTROL_CLASS =
  "px-2 py-1 text-sm rounded-md bg-input/30 border border-border outline-none focus:border-primary";

export const TEXT_INPUT_CLASS = `${CONTROL_CLASS} w-full`;
