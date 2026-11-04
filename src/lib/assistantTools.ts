// A tool call in words, for the chat: what the assistant is checking rather
// than the raw call. Names match the tools in `assistent/tools.rs`.

type ToolArguments = Record<string, unknown>;

const TOOL_LABEL: Record<string, (args: ToolArguments) => string> = {
  // The model already counts plies as the board shows them, from 1.
  get_position: (args) =>
    typeof args.ply === "number"
      ? `Looking at ply ${args.ply}`
      : "Looking at a position",
  explore_line: (args) =>
    Array.isArray(args.moves) && args.moves.length > 0
      ? `Trying ${args.moves.join(" ")}`
      : "Trying a line",
  candidate_moves: (args) =>
    Array.isArray(args.moves) && args.moves.length > 0
      ? `Asking the engine about ${args.moves.join(" ")}`
      : "Asking the engine for the best moves",
  lookup_opening: (args) =>
    typeof args.ply === "number"
      ? `Looking up the opening at ply ${args.ply}`
      : "Looking up the opening",
  get_game_review: () => "Reviewing the game",
  get_player_profile: () => "Looking at how you usually play",
};

const parseArguments = (raw: string): ToolArguments => {
  try {
    const parsed: unknown = JSON.parse(raw);
    return typeof parsed === "object" && parsed !== null
      ? (parsed as ToolArguments)
      : {};
  } catch {
    return {};
  }
};

// A tool without a label of its own shows its name and arguments as sent, so
// a new tool is still readable before it gets one.
export const toolCallLabel = (name: string, rawArguments: string): string => {
  const label = TOOL_LABEL[name];
  return label
    ? label(parseArguments(rawArguments))
    : `${name}(${rawArguments})`;
};
