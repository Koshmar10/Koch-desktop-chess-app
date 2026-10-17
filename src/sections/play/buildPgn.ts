import type { GameCreateResponse } from "../../api/bindings/GameCreateResponse";
import type { GameResult } from "../../api/bindings/GameResult";
import type { GameSummary } from "../../api/bindings/GameSummary";
import type { ReplayPosition } from "../../api/bindings/ReplayPosition";

const RESULT_TOKEN: Record<GameResult, string> = {
  WhiteWin: "1-0",
  BlackWin: "0-1",
  Draw: "1/2-1/2",
  Unfinished: "*",
};

// PGN's placeholders for a tag value that isn't known, and for a Round
// when the game wasn't part of one.
const UNKNOWN = "?";
const UNKNOWN_DATE = "????.??.??";
const NO_ROUND = "-";

const KOCH_SITE = "Koch";
const CASUAL_GAME_EVENT = "Casual Game";
const SANDBOX_EVENT = "Analysis";

const MS_PER_SECOND = 1000;

const STANDARD_START_FEN =
  "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
// The FEN fields that say who moves first, and on which move number.
const FEN_SIDE_TO_MOVE = 1;
const FEN_FULLMOVE_NUMBER = 5;

// Only each position's SAN is read, so this takes anything shaped like a
// replay's positions — `positions[0]` is the start, with no move behind it.
type PlayedPositions = readonly Pick<ReplayPosition, "san">[];
// A sandbox can start anywhere, so its start position's FEN is read too.
type SandboxPositions = readonly Pick<ReplayPosition, "san" | "fen">[];

// What every PGN is made of. The first six fields plus Round are the Seven
// Tag Roster every PGN must carry, in the order the standard fixes.
interface PgnGame {
  event: string;
  site: string;
  date: string;
  white: string;
  black: string;
  result: string;
  // Tags beyond the roster, written after it in this order. A null value
  // leaves the tag out rather than writing an empty one.
  extraTags: [name: string, value: string | null][];
  // The position `moves` are played from.
  startFen: string;
  // SAN.
  moves: string[];
}

interface MoveNumbering {
  firstMoveNumber: number;
  blackMovesFirst: boolean;
}

const numberingFrom = (fen: string): MoveNumbering => {
  const fields = fen.split(" ");
  return {
    firstMoveNumber: Number(fields[FEN_FULLMOVE_NUMBER]) || 1,
    blackMovesFirst: fields[FEN_SIDE_TO_MOVE] === "b",
  };
};

const pad = (n: number): string => n.toString().padStart(2, "0");

// PGN dates are "YYYY.MM.DD".
const pgnDate = (date: Date): string =>
  `${date.getFullYear()}.${pad(date.getMonth() + 1)}.${pad(date.getDate())}`;

// `date_played` is SQLite's UTC "YYYY-MM-DD HH:MM:SS" — read as UTC, the
// way `formatDatePlayed` does, so the PGN has the same day the game tab
// shows.
const pgnDateFromDatePlayed = (datePlayed: string | null): string => {
  if (!datePlayed) return UNKNOWN_DATE;
  const date = new Date(`${datePlayed.replace(" ", "T")}Z`);
  return isNaN(date.getTime()) ? UNKNOWN_DATE : pgnDate(date);
};

// PGN's TimeControl is "<initial>+<increment>" in seconds.
const pgnTimeControl = (initialMs: number, incrementMs: number): string =>
  `${Math.round(initialMs / MS_PER_SECOND)}+${Math.round(incrementMs / MS_PER_SECOND)}`;

// A saved game stores its clock as "<initial_ms>+<increment_ms>". Null when
// there's no clock, or it doesn't read as one.
const pgnTimeControlFromSummary = (
  timeControl: string | null,
): string | null => {
  if (!timeControl) return null;
  const [initialMs, incrementMs] = timeControl.split("+").map(Number);
  if (Number.isNaN(initialMs) || Number.isNaN(incrementMs)) return null;
  return pgnTimeControl(initialMs, incrementMs);
};

// An import with no Elo tag is saved with 0 — leave the tag out rather than
// claim the player is rated 0.
const pgnElo = (elo: number): string | null => (elo > 0 ? String(elo) : null);

// Quotes and backslashes inside a tag value must be escaped, or a name like
// `Bob "the Rook"` would end the value early.
const formatTag = (name: string, value: string): string =>
  `[${name} "${value.replace(/\\/g, "\\\\").replace(/"/g, '\\"')}"]`;

// Numbered from the start position's own move number. When Black moves
// first, that move is written "12... Nf6", as PGN expects, and White's
// next one starts the following number.
const movetext = (
  moves: string[],
  { firstMoveNumber, blackMovesFirst }: MoveNumbering,
): string =>
  moves
    .reduce<string[]>((pairs, move, idx) => {
      // Counted as if White had moved first, so even plies are White's.
      const ply = blackMovesFirst ? idx + 1 : idx;
      const moveNumber = firstMoveNumber + Math.floor(ply / 2);
      if (ply % 2 === 0) {
        pairs.push(`${moveNumber}. ${move}`);
      } else if (idx === 0) {
        pairs.push(`${moveNumber}... ${move}`);
      } else {
        pairs[pairs.length - 1] += ` ${move}`;
      }
      return pairs;
    }, [])
    .join(" ");

// The start position has no move behind it, so its `san` is null.
const sanMoves = (positions: PlayedPositions): string[] =>
  positions.flatMap((position) => position.san ?? []);

const formatPgn = (game: PgnGame): string => {
  const roster: [string, string][] = [
    ["Event", game.event],
    ["Site", game.site],
    ["Date", game.date],
    ["Round", NO_ROUND],
    ["White", game.white],
    ["Black", game.black],
    ["Result", game.result],
  ];
  // A game that doesn't start from the standard position has to say where
  // it does start, or nothing reading it can play its moves.
  const setUpTags: [string, string][] =
    game.startFen === STANDARD_START_FEN
      ? []
      : [
          ["SetUp", "1"],
          ["FEN", game.startFen],
        ];
  const tags = [...roster, ...setUpTags, ...game.extraTags]
    .flatMap(([name, value]) => (value === null ? [] : formatTag(name, value)))
    .join("\n");

  const moves = movetext(game.moves, numberingFrom(game.startFen));
  const line = moves ? `${moves} ${game.result}` : game.result;

  return `${tags}\n\n${line}\n`;
};

// A koch game just played. Everything a PGN needs — players, time
// control, move history, result — is already in GameCreateResponse, so
// this needs no round trip to the backend.
export const buildLiveGamePgn = (game: GameCreateResponse): string =>
  formatPgn({
    event: CASUAL_GAME_EVENT,
    site: KOCH_SITE,
    date: pgnDate(new Date()),
    white: game.white_player.name,
    black: game.black_player.name,
    result: RESULT_TOKEN[game.state.result],
    extraTags: [
      ["WhiteElo", String(game.white_player.elo)],
      ["BlackElo", String(game.black_player.elo)],
      [
        "TimeControl",
        pgnTimeControl(
          game.time_control.initial_ms,
          game.time_control.increment_ms,
        ),
      ],
      ["Opening", game.state.opening_name],
    ],
    startFen: STANDARD_START_FEN,
    moves: game.state.move_history,
  });

// A saved game opened in the analyzer — a finished koch game or an import.
// An import's own Event and Site aren't kept when it's saved, so those come
// out unknown rather than claiming it was played on Koch.
export const buildSavedGamePgn = ({
  summary,
  positions,
}: {
  summary: GameSummary;
  positions: PlayedPositions;
}): string => {
  const isKochGame = summary.source === "koch";
  return formatPgn({
    event: isKochGame ? CASUAL_GAME_EVENT : UNKNOWN,
    site: isKochGame ? KOCH_SITE : UNKNOWN,
    date: pgnDateFromDatePlayed(summary.date_played),
    white: summary.white_player,
    black: summary.black_player,
    // Already a PGN token — the backend stores `GameResult::as_str()`.
    result: summary.result,
    extraTags: [
      ["WhiteElo", pgnElo(summary.white_elo)],
      ["BlackElo", pgnElo(summary.black_elo)],
      ["TimeControl", pgnTimeControlFromSummary(summary.time_control)],
      ["Opening", summary.opening_name],
    ],
    // A saved game is always replayed from the standard start.
    startFen: STANDARD_START_FEN,
    moves: sanMoves(positions),
  });
};

// The analyzer's sandbox: moves made by hand, with no players and no
// result, from whatever position it was started on.
export const buildSandboxPgn = (positions: SandboxPositions): string =>
  formatPgn({
    event: SANDBOX_EVENT,
    site: KOCH_SITE,
    date: pgnDate(new Date()),
    white: UNKNOWN,
    black: UNKNOWN,
    result: RESULT_TOKEN.Unfinished,
    extraTags: [],
    startFen: positions[0]?.fen ?? STANDARD_START_FEN,
    moves: sanMoves(positions),
  });
