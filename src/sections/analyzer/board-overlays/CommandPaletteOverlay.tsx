import { useState, type KeyboardEvent } from "react";
import { BOARD_SIZE_PX } from "../../../components/chessboard/lib/constants";
import DialogCard, { NARROW_DIALOG_PX } from "../../../components/DialogCard";
import EmptyNote from "../../../components/EmptyNote";
import {
  filterCommands,
  shortcutLabel,
  type PaletteCommand,
} from "../../../lib/commandPalette";

// Half the board, so the palette never covers all of it.
const LIST_MAX_HEIGHT_PX = BOARD_SIZE_PX / 2;
const HIDDEN_SCROLLBAR_CLASS =
  "[scrollbar-width:none] [&::-webkit-scrollbar]:hidden";
const SELECTED_ROW_CLASS = "bg-primary/30 text-foreground";
const ROW_CLASS = "text-card-foreground/80 hover:bg-primary/15";

interface CommandPaletteOverlayProps {
  commands: PaletteCommand[];
  onClose: () => void;
}

// Type to narrow the list, ↑ and ↓ to pick, Enter to run, Esc to close.
// Running a command closes the palette first, so a command that opens
// another overlay isn't left underneath this one.
const CommandPaletteOverlay = ({
  commands,
  onClose,
}: CommandPaletteOverlayProps) => {
  const [query, setQuery] = useState("");
  const [selected, setSelected] = useState(0);
  const shown = filterCommands(commands, query);

  const run = (command: PaletteCommand) => {
    onClose();
    command.run();
  };

  const onKeyDown = (event: KeyboardEvent<HTMLInputElement>) => {
    if (event.key === "Escape") {
      event.preventDefault();
      onClose();
    } else if (event.key === "ArrowDown" || event.key === "ArrowUp") {
      event.preventDefault();
      const step = event.key === "ArrowDown" ? 1 : -1;
      // Wraps round, so the last command is one ↑ away from the top.
      setSelected((index) => (index + step + shown.length) % shown.length);
    } else if (event.key === "Enter" && shown[selected]) {
      event.preventDefault();
      run(shown[selected]);
    }
  };

  return (
    // Titled so the close button has a row of its own, above the input
    // rather than over it.
    <DialogCard title="Commands" onClose={onClose} widthPx={NARROW_DIALOG_PX}>
      <input
        type="text"
        aria-label="Search commands"
        placeholder="Type a command"
        value={query}
        onChange={(event) => {
          setQuery(event.target.value);
          setSelected(0);
        }}
        onKeyDown={onKeyDown}
        autoFocus
        className="w-full rounded-md bg-secondary/50 px-2 py-1.5 text-sm"
      />
      {shown.length === 0 ? (
        <EmptyNote>No command matches</EmptyNote>
      ) : (
        // Scrolls inside the card, so a long list leaves the board visible
        // around it.
        <ul
          role="listbox"
          className={`flex flex-col overflow-y-auto ${HIDDEN_SCROLLBAR_CLASS}`}
          style={{ maxHeight: LIST_MAX_HEIGHT_PX }}
        >
          {shown.map((command, index) => (
            <li
              key={command.id}
              role="option"
              aria-selected={index === selected}
              // Keeps the row picked with ↑ and ↓ in view.
              ref={
                index === selected
                  ? (row) => row?.scrollIntoView({ block: "nearest" })
                  : undefined
              }
            >
              <button
                type="button"
                onClick={() => run(command)}
                onMouseEnter={() => setSelected(index)}
                className={`flex w-full cursor-pointer items-center justify-between gap-3 rounded-md px-2 py-1.5 text-left text-sm ${
                  index === selected ? SELECTED_ROW_CLASS : ROW_CLASS
                }`}
              >
                {command.label}
                {command.shortcuts[0] && (
                  <kbd className="shrink-0 rounded border border-border/80 px-1.5 font-mono text-xs text-muted-foreground">
                    {shortcutLabel(command.shortcuts[0])}
                  </kbd>
                )}
              </button>
            </li>
          ))}
        </ul>
      )}
    </DialogCard>
  );
};

export default CommandPaletteOverlay;
