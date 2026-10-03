import type { PlacedPiece } from "../../components/chessboard/lib/types";
import type { GameSummary } from "../../api/bindings/GameSummary";
import type { MoveQuality } from "../../api/bindings/MoveQuality";
import type { PositionFindings } from "../../api/bindings/PositionFindings";
import type { SideAccuracy } from "../../api/bindings/SideAccuracy";
import type { EngineSnapshot } from "./types";

/**
 * The game the analyzer is pretending to have open. Shaped as the real
 * `GameSummary` that History already lists and that KOCH-12 will hand
 * this screen, so the game tab needs no rework when loading arrives —
 * only its source changes.
 */
export const MOCK_GAME: GameSummary = {
  game_id: 1,
  game_hash: "mock-ruy-lopez",
  date_played: "2026-09-12 19:41:00",
  white_player: "Petru",
  black_player: "Stockfish 17",
  white_elo: 1640,
  black_elo: 1800,
  result: "1-0",
  opening_id: null,
  opening_name: "Ruy Lopez, Closed Defence",
  time_control: "600+0",
  source: "koch",
  human_color: "white",
  has_analysis: true,
  partial_import: false,
};

// Stand-ins until KOCH-10 streams the real thing. Kept in the shape the
// backend will emit, not a convenient-for-React shape, so wiring the
// session in later is a swap of the source and nothing else.

/**
 * The start position at a plausible search depth. Real-ish numbers rather
 * than round ones: a bar and a line list both look fine on `+1.00` and
 * fall apart on `+0.28`, which is the case worth seeing while building.
 */
export const MOCK_ENGINE_SNAPSHOT: EngineSnapshot = {
  position_key: "sandbox",
  depth: 24,
  multi_pv: 4,
  pv_lines: [
    {
      score: { kind: "cp", centipawns: 28 },
      moves: ["e2e4", "e7e5", "g1f3", "b8c6", "f1b5"],
    },
    {
      score: { kind: "cp", centipawns: 24 },
      moves: ["d2d4", "g8f6", "c2c4", "e7e6", "g1f3"],
    },
    {
      score: { kind: "cp", centipawns: 19 },
      moves: ["g1f3", "d7d5", "d2d4", "g8f6", "c2c4"],
    },
    {
      score: { kind: "cp", centipawns: 17 },
      moves: ["c2c4", "e7e5", "b1c3", "g8f6", "g1f3"],
    },
  ],
};

/**
 * A Ruy Lopez opening, in SAN, purely so the ply scrubber has a real move
 * list to walk. The board stays on the starting position for now — moving
 * it in step with this is KOCH-12, not part of the shell.
 */
export const MOCK_MOVES: string[] = [
  "e4",
  "e5",
  "Nf3",
  "Nc6",
  "Bb5",
  "a6",
  "Ba4",
  "Nf6",
  "O-O",
  "Be7",
  "Re1",
  "b5",
  "Bb3",
  "d6",
  "c3",
  "O-O",
];

/**
 * A grade for every move in MOCK_MOVES, both sides — the analysis grades
 * both now. Illustrative: White's column is chosen to show every colour
 * once, not a verdict on these moves (10.Re1 is no mistake).
 */
export const MOCK_QUALITIES: (MoveQuality | null)[] = [
  "Brilliant", // e4
  "Brilliant", // e5
  "Great", // Nf3
  "Great", // Nc6
  "Excellent", // Bb5
  "Good", // a6
  "Good", // Ba4
  "Excellent", // Nf6
  "Inaccuracy", // O-O
  "Great", // Be7
  "Mistake", // Re1
  "Inaccuracy", // b5
  "Blunder", // Bb3
  "Good", // d6
  "Great", // c3
  "Excellent", // O-O
];

/** Each side's accuracy over MOCK_MOVES, in line with the grades above. */
export const MOCK_WHITE_ACCURACY: SideAccuracy = {
  accuracy_percent: 78.4,
  average_centipawn_loss: 41,
};
export const MOCK_BLACK_ACCURACY: SideAccuracy = {
  accuracy_percent: 91.2,
  average_centipawn_loss: 17,
};

/**
 * Remaining clock after each ply of MOCK_MOVES, from a 10-minute game.
 * Illustrative: nothing in the current backend carries per-move clocks
 * yet — `GameStateView.move_history` is SAN only and `GameAnalysis` has
 * the human's move *times*, not the clock they left behind. The move
 * timeline drops the column when it gets none, so this is what the
 * populated version looks like.
 */
export const MOCK_CLOCKS: string[] = [
  "9:58",
  "9:57",
  "9:51",
  "9:49",
  "9:45",
  "9:40",
  "9:33",
  "9:26",
  "9:20",
  "9:11",
  "9:08",
  "8:59",
  "8:55",
  "8:44",
  "8:40",
  "8:30",
];

// The position MOCK_MOVES actually reaches, so the board, the move count
// and every id the findings below reference describe the same chess. Ids
// follow STARTING_POSITION's scheme (black back rank 0-7, black pawns
// 8-15, white pawns 16-23, white back rank 24-31), carried through the
// moves — so id 1 is the knight that began on b8 and now stands on c6.
export const MOCK_PIECES: PlacedPiece[] = [
  // Black
  { id: 0, kind: "rook", color: "black", square: { rank: 0, file: 0 } },
  { id: 2, kind: "bishop", color: "black", square: { rank: 0, file: 2 } },
  { id: 3, kind: "queen", color: "black", square: { rank: 0, file: 3 } },
  { id: 7, kind: "rook", color: "black", square: { rank: 0, file: 5 } },
  { id: 4, kind: "king", color: "black", square: { rank: 0, file: 6 } },
  { id: 5, kind: "bishop", color: "black", square: { rank: 1, file: 4 } },
  { id: 1, kind: "knight", color: "black", square: { rank: 2, file: 2 } },
  { id: 6, kind: "knight", color: "black", square: { rank: 2, file: 5 } },
  { id: 8, kind: "pawn", color: "black", square: { rank: 2, file: 0 } },
  { id: 9, kind: "pawn", color: "black", square: { rank: 3, file: 1 } },
  { id: 10, kind: "pawn", color: "black", square: { rank: 1, file: 2 } },
  { id: 11, kind: "pawn", color: "black", square: { rank: 2, file: 3 } },
  { id: 12, kind: "pawn", color: "black", square: { rank: 3, file: 4 } },
  { id: 13, kind: "pawn", color: "black", square: { rank: 1, file: 5 } },
  { id: 14, kind: "pawn", color: "black", square: { rank: 1, file: 6 } },
  { id: 15, kind: "pawn", color: "black", square: { rank: 1, file: 7 } },
  // White
  { id: 24, kind: "rook", color: "white", square: { rank: 7, file: 0 } },
  { id: 25, kind: "knight", color: "white", square: { rank: 7, file: 1 } },
  { id: 26, kind: "bishop", color: "white", square: { rank: 7, file: 2 } },
  { id: 27, kind: "queen", color: "white", square: { rank: 7, file: 3 } },
  { id: 31, kind: "rook", color: "white", square: { rank: 7, file: 4 } },
  { id: 28, kind: "king", color: "white", square: { rank: 7, file: 6 } },
  { id: 29, kind: "bishop", color: "white", square: { rank: 5, file: 1 } },
  { id: 30, kind: "knight", color: "white", square: { rank: 5, file: 5 } },
  { id: 16, kind: "pawn", color: "white", square: { rank: 6, file: 0 } },
  { id: 17, kind: "pawn", color: "white", square: { rank: 6, file: 1 } },
  { id: 18, kind: "pawn", color: "white", square: { rank: 5, file: 2 } },
  { id: 19, kind: "pawn", color: "white", square: { rank: 6, file: 3 } },
  { id: 20, kind: "pawn", color: "white", square: { rank: 4, file: 4 } },
  { id: 21, kind: "pawn", color: "white", square: { rank: 6, file: 5 } },
  { id: 22, kind: "pawn", color: "white", square: { rank: 6, file: 6 } },
  { id: 23, kind: "pawn", color: "white", square: { rank: 6, file: 7 } },
];

const SQUARE_COUNT = 64;

/**
 * Who covers which squares in the position above — centre and flanks
 * only, where a Closed Ruy is actually fought over, rather than all 64.
 * Each entry names the real covering pieces (by id) for its square:
 * d4 is hit by White's c3 pawn and f3 knight and by Black's e5 pawn and
 * c6 knight, so it nets to zero and reads as contested.
 *
 * Built from rosters rather than written as raw numbers so the sign
 * convention can't be got wrong by hand: `square_threats` is derived as
 * Black's count minus White's, exactly as koch-engine's `threat_map`
 * does (White −1, Black +1 per coverer).
 */
const CONTROL_FIXTURE: {
  rank: number;
  file: number;
  white: number[];
  black: number[];
}[] = [
  { rank: 4, file: 3, white: [18, 30], black: [12, 1] }, // d4 — contested
  { rank: 4, file: 2, white: [29], black: [9] }, // c4 — contested
  { rank: 4, file: 1, white: [18], black: [1] }, // b4 — contested
  { rank: 4, file: 4, white: [31], black: [6] }, // e4 — contested
  { rank: 3, file: 3, white: [20, 29], black: [6] }, // d5 — White
  { rank: 3, file: 5, white: [20], black: [] }, // f5 — White
  { rank: 3, file: 6, white: [30], black: [] }, // g5 — White
  { rank: 4, file: 7, white: [30], black: [] }, // h4 — White
  { rank: 3, file: 4, white: [30], black: [11, 1] }, // e5 — Black
  { rank: 4, file: 5, white: [], black: [12] }, // f4 — Black
  { rank: 4, file: 6, white: [], black: [6, 2] }, // g4 — Black
];

const squareIndex = (rank: number, file: number) => rank * 8 + file;

const controlFixture = () => {
  const square_threats = Array.from({ length: SQUARE_COUNT }, () => 0);
  const attackers: number[][] = Array.from({ length: SQUARE_COUNT }, () => []);
  const defenders: number[][] = Array.from({ length: SQUARE_COUNT }, () => []);
  for (const { rank, file, white, black } of CONTROL_FIXTURE) {
    const index = squareIndex(rank, file);
    square_threats[index] = black.length - white.length;
    attackers[index] = white;
    defenders[index] = black;
  }
  return { square_threats, attackers, defenders };
};

/**
 * Black's best reply if White passed — what the threat probe (KOCH-14)
 * will report. ...Na5 hitting the b3 bishop is *the* Closed Ruy threat:
 * it's why White so often spends a tempo on Bc2 in this structure.
 */
export const MOCK_THREAT_MOVE = "c6a5";

/**
 * Findings for the position above.
 *
 * The pin is real geometry — the b3 bishop looks down the b3-g8 diagonal
 * at f7, with Black's king behind it — and the king safety numbers are
 * plausible for a Closed Ruy.
 *
 * The fork and the pawn weaknesses are **fixtures**, not the truth about
 * the position: a Closed Ruy has no fork, and both sides hold all eight
 * files, so nothing there is really passed or isolated. They're set so
 * every row type renders while the panel is being built. The fixture is
 * at least self-consistent — White's island list and `unoccupied_files`
 * agree with the isolated a2 pawn rather than contradicting it on screen
 * — but it describes fewer pawns than the board shows. Empty every list
 * to see the panel's empty states.
 *
 * Square control comes from `CONTROL_FIXTURE` above — real coverers for
 * the squares that matter, not all 64.
 */
export const MOCK_POSITION_FINDINGS: PositionFindings = {
  ...controlFixture(),
  hanging_squares: [{ rank: 3, file: 4 }],
  pins: [
    {
      pinner_piece_id: 29, // Bb3
      pinned_piece_id: 13, // the f7 pawn
      pin_target_id: 4, // Kg8
      squares: [
        { rank: 4, file: 2 },
        { rank: 3, file: 3 },
        { rank: 2, file: 4 },
        { rank: 1, file: 5 },
        { rank: 0, file: 6 },
      ],
    },
  ],
  forks: [{ forker_id: 30, forked_ids: [12, 9] }],
  white_pawn_structure: {
    color: "white",
    // a2 stands alone with the b-file empty beside it, which is what
    // makes it isolated — so the islands and unoccupied_files below have
    // to show that gap too, or the panel would claim one connected
    // island next to a pawn marked isolated.
    pawn_islands: [[16], [18, 19, 20, 21, 22, 23]],
    backward_pawn_ids: [19],
    passed_pawn_ids: [],
    isolated_pawn_ids: [16],
    doubled_pawn_ids: [],
    unoccupied_files: [false, true, false, false, false, false, false, false],
  },
  black_pawn_structure: {
    color: "black",
    pawn_islands: [[8, 9, 10, 11, 12, 13, 14, 15]],
    backward_pawn_ids: [10],
    // b5, the most advanced pawn on the queenside — a passed pawn doesn't
    // need a gap in the structure, so this one costs the island list
    // nothing.
    passed_pawn_ids: [9],
    isolated_pawn_ids: [],
    doubled_pawn_ids: [],
    unoccupied_files: [false, false, false, false, false, false, false, false],
  },
  white_king_safety: {
    color: "white",
    score: 4,
    danger: "Safe",
    shield_penalty: 0,
    storm_penalty: 4,
    attack_penalty: 0,
    missing_shield_files: [],
    advanced_shield_pawn_ids: [],
    storming_pawn_ids: [9],
    attacking_piece_ids: [],
  },
  black_king_safety: {
    color: "black",
    score: 22,
    danger: "Uneasy",
    shield_penalty: 0,
    storm_penalty: 8,
    attack_penalty: 14,
    missing_shield_files: [],
    advanced_shield_pawn_ids: [],
    storming_pawn_ids: [20],
    attacking_piece_ids: [29, 30, 27],
  },
};
