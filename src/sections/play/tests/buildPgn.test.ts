import { describe, expect, it } from "vitest";
import type { GameCreateResponse } from "../../../api/bindings/GameCreateResponse";
import type { GameStateView } from "../../../api/bindings/GameStateView";
import type { GameSummary } from "../../../api/bindings/GameSummary";
import {
  buildLiveGamePgn,
  buildSandboxPgn,
  buildSavedGamePgn,
} from "../buildPgn";

const tagsOf = (pgn: string): string[] => pgn.split("\n\n")[0].split("\n");
const movetextOf = (pgn: string): string => pgn.split("\n\n")[1].trim();

const STANDARD_START =
  "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
// After 1. e4 e5 2. Nf3: Black to move, on move 2.
const BLACK_TO_MOVE =
  "rnbqkbnr/pppp1ppp/8/4p3/4P3/5N2/PPPP1PPP/RNBQKB1R b KQkq - 1 2";

// As a replay lists them: the start position first, with no move behind it.
// Only the start's FEN is ever read, so the rest go without one.
const positionsFor = (moves: string[], startFen = STANDARD_START) => [
  { san: null, fen: startFen },
  ...moves.map((san) => ({ san, fen: "" })),
];

const summary = (overrides: Partial<GameSummary> = {}): GameSummary => ({
  game_id: 1,
  game_hash: "abc",
  date_played: "2026-10-05 12:00:00",
  white_player: "Petru",
  black_player: "Koch 1500",
  white_elo: 1200,
  black_elo: 1500,
  result: "1-0",
  opening_id: null,
  opening_name: "Italian Game",
  time_control: "300000+3000",
  source: "koch",
  human_color: "white",
  has_analysis: false,
  partial_import: false,
  ...overrides,
});

describe("buildLiveGamePgn", () => {
  const game: GameCreateResponse = {
    white_player: { name: "Petru", elo: 1200 },
    black_player: { name: "Koch 1500", elo: 1500 },
    time_control: { initial_ms: 300_000, increment_ms: 3_000 },
    state: {
      result: "BlackWin",
      move_history: ["f3", "e5", "g4", "Qh4#"],
      opening_name: null,
    } as GameStateView,
  };

  it("writes the players, clock and result of a just-played game", () => {
    const tags = tagsOf(buildLiveGamePgn(game));

    expect(tags).toContain('[White "Petru"]');
    expect(tags).toContain('[BlackElo "1500"]');
    expect(tags).toContain('[TimeControl "300+3"]');
    expect(tags).toContain('[Result "0-1"]');
  });

  it("leaves out a tag it has no value for", () => {
    expect(buildLiveGamePgn(game)).not.toContain("[Opening");
  });

  it("numbers the moves and ends on the result", () => {
    expect(movetextOf(buildLiveGamePgn(game))).toBe("1. f3 e5 2. g4 Qh4# 0-1");
  });
});

describe("buildSavedGamePgn", () => {
  it("opens with the Seven Tag Roster, in order", () => {
    const pgn = buildSavedGamePgn({
      summary: summary(),
      positions: positionsFor([]),
    });

    expect(tagsOf(pgn).slice(0, 7)).toEqual([
      '[Event "Casual Game"]',
      '[Site "Koch"]',
      '[Date "2026.10.05"]',
      '[Round "-"]',
      '[White "Petru"]',
      '[Black "Koch 1500"]',
      '[Result "1-0"]',
    ]);
  });

  it("converts the stored millisecond clock to PGN seconds", () => {
    const pgn = buildSavedGamePgn({
      summary: summary(),
      positions: positionsFor([]),
    });

    expect(tagsOf(pgn)).toContain('[TimeControl "300+3"]');
    expect(tagsOf(pgn)).toContain('[Opening "Italian Game"]');
  });

  it("skips the start position and ends an odd move list on the result", () => {
    const pgn = buildSavedGamePgn({
      summary: summary(),
      positions: positionsFor(["e4", "e5", "Qh5"]),
    });

    expect(movetextOf(pgn)).toBe("1. e4 e5 2. Qh5 1-0");
  });

  it("doesn't claim an import was played on Koch, or rated 0", () => {
    const pgn = buildSavedGamePgn({
      summary: summary({
        source: "pgn",
        white_elo: 0,
        date_played: null,
        time_control: null,
      }),
      positions: positionsFor([]),
    });
    const tags = tagsOf(pgn);

    expect(tags).toContain('[Event "?"]');
    expect(tags).toContain('[Site "?"]');
    expect(tags).toContain('[Date "????.??.??"]');
    expect(pgn).not.toContain("[WhiteElo");
    expect(pgn).not.toContain("[TimeControl");
  });

  it("escapes quotes and backslashes in tag values", () => {
    const pgn = buildSavedGamePgn({
      summary: summary({ white_player: 'Bob "the Rook" \\o/' }),
      positions: positionsFor([]),
    });

    expect(tagsOf(pgn)).toContain('[White "Bob \\"the Rook\\" \\\\o/"]');
  });
});

describe("buildSandboxPgn", () => {
  it("has unknown players and an unfinished result", () => {
    const tags = tagsOf(buildSandboxPgn(positionsFor(["d4"])));

    expect(tags).toContain('[Event "Analysis"]');
    expect(tags).toContain('[White "?"]');
    expect(tags).toContain('[Black "?"]');
    expect(tags).toContain('[Result "*"]');
  });

  it("writes the moves made so far", () => {
    expect(movetextOf(buildSandboxPgn(positionsFor(["d4", "d5", "c4"])))).toBe(
      "1. d4 d5 2. c4 *",
    );
  });

  it("is just the result before any move is made", () => {
    expect(movetextOf(buildSandboxPgn(positionsFor([])))).toBe("*");
  });

  it("leaves out SetUp and FEN when it starts from the standard position", () => {
    expect(buildSandboxPgn(positionsFor(["e4"]))).not.toContain("[FEN");
  });

  it("says where it starts when that isn't the standard position", () => {
    const tags = tagsOf(buildSandboxPgn(positionsFor([], BLACK_TO_MOVE)));

    expect(tags).toContain('[SetUp "1"]');
    expect(tags).toContain(`[FEN "${BLACK_TO_MOVE}"]`);
  });

  it("numbers from the start position, with Black's first move as n...", () => {
    const pgn = buildSandboxPgn(
      positionsFor(["Nc6", "Bb5", "a6"], BLACK_TO_MOVE),
    );

    expect(movetextOf(pgn)).toBe("2... Nc6 3. Bb5 a6 *");
  });
});
