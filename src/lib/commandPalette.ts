// A key press that runs a command. Matched against a `KeyboardEvent`.
export interface Shortcut {
  // `KeyboardEvent.key`, lower-cased: "f", "arrowleft", "home", "/".
  key: string;
  // Ctrl, or Cmd on a Mac. A shortcut never matches with Alt held.
  ctrl?: boolean;
  // Left out, Shift doesn't matter: some layouts need it to type "/".
  shift?: boolean;
  // Held keys repeat; false for one where a burst would do harm.
  repeat?: boolean;
  // Also works with focus in a text field. Only for keys that type
  // nothing there, like Esc closing the dialog the field is in.
  whileTyping?: boolean;
}

// One thing the screen can do, run from the command palette or by its
// shortcuts. Built by the screen from the actions it already has, so
// neither offers something the screen can't do right now.
export interface PaletteCommand {
  id: string;
  label: string;
  // The first is the one shown beside the command, so the palette also
  // teaches the keys. Empty for a command with none.
  shortcuts: Shortcut[];
  run: () => void;
}

// How a key is written where its `key` value isn't how it's printed.
const KEY_LABEL: Record<string, string> = {
  arrowleft: "←",
  arrowright: "→",
  arrowup: "↑",
  arrowdown: "↓",
  home: "Home",
  end: "End",
  escape: "Esc",
};

// "Ctrl+Z", "Shift+←", "F".
export const shortcutLabel = (shortcut: Shortcut): string =>
  [
    shortcut.ctrl && "Ctrl",
    shortcut.shift && "Shift",
    KEY_LABEL[shortcut.key] ?? shortcut.key.toUpperCase(),
  ]
    .filter(Boolean)
    .join("+");

// The parts of a key press a shortcut looks at.
export type KeyPress = Pick<
  KeyboardEvent,
  "key" | "ctrlKey" | "metaKey" | "shiftKey" | "altKey" | "repeat"
>;

export const matchesShortcut = (press: KeyPress, shortcut: Shortcut) =>
  press.key.toLowerCase() === shortcut.key &&
  (press.ctrlKey || press.metaKey) === Boolean(shortcut.ctrl) &&
  !press.altKey &&
  (shortcut.shift === undefined || press.shiftKey === shortcut.shift) &&
  !(press.repeat && shortcut.repeat === false);

// The command a key press runs: the first in the list with a matching
// shortcut, so where two share a key, the earlier one wins. With focus in
// a text field, only shortcuts meant for that count.
export const commandForKey = (
  commands: readonly PaletteCommand[],
  press: KeyPress,
  isTyping: boolean,
): PaletteCommand | null =>
  commands.find((command) =>
    command.shortcuts.some(
      (shortcut) =>
        (!isTyping || shortcut.whileTyping) && matchesShortcut(press, shortcut),
    ),
  ) ?? null;

// The commands whose label has every word typed, in any order, ignoring
// case: "flip" and "board fl" both find "Flip the board". Those starting
// with the first word typed come first; otherwise the order is kept.
export const filterCommands = (
  commands: readonly PaletteCommand[],
  query: string,
): PaletteCommand[] => {
  const words = query.toLowerCase().split(/\s+/).filter(Boolean);
  if (words.length === 0) return [...commands];
  const matches = commands.filter((command) => {
    const label = command.label.toLowerCase();
    return words.every((word) => label.includes(word));
  });
  const startsWithFirst = (command: PaletteCommand) =>
    command.label.toLowerCase().startsWith(words[0]);
  return [
    ...matches.filter(startsWithFirst),
    ...matches.filter((command) => !startsWithFirst(command)),
  ];
};
