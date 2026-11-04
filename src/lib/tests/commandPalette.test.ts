import { describe, expect, it } from "vitest";
import {
  commandForKey,
  filterCommands,
  matchesShortcut,
  shortcutLabel,
  type KeyPress,
  type PaletteCommand,
  type Shortcut,
} from "../commandPalette";

const command = (
  label: string,
  shortcuts: Shortcut[] = [],
): PaletteCommand => ({
  id: label,
  label,
  shortcuts,
  run: () => {},
});

const press = (key: string, modifiers: Partial<KeyPress> = {}): KeyPress => ({
  key,
  ctrlKey: false,
  metaKey: false,
  shiftKey: false,
  altKey: false,
  repeat: false,
  ...modifiers,
});

describe("shortcutLabel", () => {
  it("writes modifiers first and keys as they're printed", () => {
    expect(shortcutLabel({ key: "z", ctrl: true })).toBe("Ctrl+Z");
    expect(shortcutLabel({ key: "arrowleft", shift: true })).toBe("Shift+←");
    expect(shortcutLabel({ key: "escape" })).toBe("Esc");
    expect(shortcutLabel({ key: "/" })).toBe("/");
  });
});

describe("matchesShortcut", () => {
  it("matches the key whatever its case", () => {
    expect(matchesShortcut(press("F"), { key: "f" })).toBe(true);
  });

  it("takes Cmd for Ctrl, and needs it only when asked for", () => {
    const undo = { key: "z", ctrl: true };
    expect(matchesShortcut(press("z", { metaKey: true }), undo)).toBe(true);
    expect(matchesShortcut(press("z"), undo)).toBe(false);
    expect(matchesShortcut(press("f", { ctrlKey: true }), { key: "f" })).toBe(
      false,
    );
  });

  it("checks Shift only when the shortcut says", () => {
    const step = { key: "arrowleft", shift: false };
    expect(matchesShortcut(press("ArrowLeft", { shiftKey: true }), step)).toBe(
      false,
    );
    expect(matchesShortcut(press("/", { shiftKey: true }), { key: "/" })).toBe(
      true,
    );
  });

  it("never matches under Alt", () => {
    expect(matchesShortcut(press("f", { altKey: true }), { key: "f" })).toBe(
      false,
    );
  });

  it("ignores a held key's repeats where asked to", () => {
    const held = press("z", { ctrlKey: true, repeat: true });
    expect(matchesShortcut(held, { key: "z", ctrl: true, repeat: false })).toBe(
      false,
    );
    expect(matchesShortcut(held, { key: "z", ctrl: true })).toBe(true);
  });
});

describe("commandForKey", () => {
  const clear = command("Clear", [{ key: "escape" }]);
  const close = command("Close", [{ key: "escape", whileTyping: true }]);

  it("picks the first command with a matching shortcut", () => {
    expect(commandForKey([close, clear], press("Escape"), false)).toBe(close);
    expect(commandForKey([clear, close], press("Escape"), false)).toBe(clear);
  });

  it("while typing, only counts shortcuts meant for it", () => {
    expect(commandForKey([clear, close], press("Escape"), true)).toBe(close);
    expect(commandForKey([clear], press("Escape"), true)).toBe(null);
  });

  it("is null for a key no command has", () => {
    expect(commandForKey([clear], press("q"), false)).toBe(null);
  });
});

const COMMANDS = [
  command("Go to the start"),
  command("Flip the board"),
  command("Clear the board's overlays"),
  command("Board theme"),
];

const labels = (commands: PaletteCommand[]) => commands.map((c) => c.label);

describe("filterCommands", () => {
  it("keeps every command, in order, for an empty query", () => {
    expect(labels(filterCommands(COMMANDS, "  "))).toEqual(labels(COMMANDS));
  });

  it("matches every word typed, in any order and case", () => {
    expect(labels(filterCommands(COMMANDS, "BOARD fl"))).toEqual([
      "Flip the board",
    ]);
  });

  it("puts labels starting with the first word first", () => {
    expect(labels(filterCommands(COMMANDS, "board"))).toEqual([
      "Board theme",
      "Flip the board",
      "Clear the board's overlays",
    ]);
  });

  it("finds nothing for words no label has", () => {
    expect(filterCommands(COMMANDS, "resign")).toEqual([]);
  });
});
