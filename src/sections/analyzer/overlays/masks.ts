import type { Fork } from "../../../api/bindings/Fork";
import type { KingSafety } from "../../../api/bindings/KingSafety";
import type { PawnStructure } from "../../../api/bindings/PawnStructure";
import type { PieceColor } from "../../../api/bindings/PieceColor";
import type { Pin } from "../../../api/bindings/Pin";
import type { PositionFindings } from "../../../api/bindings/PositionFindings";
import type { Square } from "../../../api/bindings/Square";
import {
  TONE_ARROW_COLOR,
  TONE_SQUARE_COLOR,
} from "../../../components/chessboard/lib/highlightTones";
import type {
  ArrowData,
  HighlightTone,
  OverlayMask,
  PlacedPiece,
  SquareMark,
} from "../../../components/chessboard/lib/types";
import { parseUciMove } from "../../../components/chessboard/lib/uci";
import { controlTints } from "../panel/position/controlTint";

// Paint order, lowest first. The whole-board shading goes underneath, so
// anything lit on top of it still stands out; then the slow-moving
// structure of the position; then the tactics; and the engine's moves on
// top, since a suggestion is the thing you're about to act on. Spaced by
// ten so a new concept can slot in between without renumbering the rest.
export const CONTROL_PRIORITY = 0;
export const PAWN_PRIORITY = 10;
export const KING_SAFETY_PRIORITY = 20;
export const TACTIC_PRIORITY = 30;
export const ENGINE_MOVE_PRIORITY = 40;

export const PAWN_FLAG_KEYS = [
  "passed_pawn_ids",
  "backward_pawn_ids",
  "isolated_pawn_ids",
  "doubled_pawn_ids",
] as const;

export type PawnFlagKey = (typeof PAWN_FLAG_KEYS)[number];

/**
 * Every mask's id, in one place. The builders below stamp them and the
 * panel rows use them to say what they toggle, so the two can't drift
 * apart — a row whose id doesn't match its mask's toggles nothing, and
 * nothing says so.
 *
 * Named for how a call reads — `maskIdFor.king("white")`, the mask id for
 * White's king — and so it doesn't share a name with the rows' `maskId`
 * prop, which holds one id rather than the ways to make them.
 */
export const maskIdFor = {
  control: "control",
  bestMove: "engine:best",
  threat: "engine:threat",
  king: (color: PieceColor) => `king:${color}`,
  pawns: (color: PieceColor, flag: PawnFlagKey) => `pawns:${color}:${flag}`,
  pin: (pin: Pin) => `pin:${pin.pinner_piece_id}-${pin.pinned_piece_id}`,
  // The victims too, not just the forker: the same knight forking two
  // other pieces a few moves later is a different fork, and shouldn't
  // light up as though it were the one you picked. Sorted so the order
  // the backend lists them in can't change the id.
  fork: (fork: Fork) =>
    `fork:${fork.forker_id}:${[...fork.forked_ids].sort((a, b) => a - b).join(",")}`,
};

const squareOf = (id: number, pieces: PlacedPiece[]): Square | undefined =>
  pieces.find((p) => p.id === id)?.square;

/**
 * Where the named pieces stand, filled in one tone — skipping any id this
 * position doesn't hold, which happens whenever the findings and the
 * rendered position are a ply apart.
 */
const marksOf = (
  ids: number[],
  pieces: PlacedPiece[],
  tone: HighlightTone,
): SquareMark[] =>
  ids
    .map((id) => squareOf(id, pieces))
    .filter((square): square is Square => square !== undefined)
    .map((square) => ({ square, color: TONE_SQUARE_COLOR[tone] }));

/** An arrow between two pieces, or nothing if either has left the board. */
const arrowBetween = (
  fromId: number,
  toId: number,
  pieces: PlacedPiece[],
  tone: HighlightTone,
): ArrowData[] => {
  const from = squareOf(fromId, pieces);
  const to = squareOf(toId, pieces);
  if (!from || !to) return [];
  return [
    {
      from: [from.rank, from.file],
      to: [to.rank, to.file],
      color: TONE_ARROW_COLOR[tone],
      type: "finding",
    },
  ];
};

export const controlMask = (findings: PositionFindings): OverlayMask => ({
  id: maskIdFor.control,
  priority: CONTROL_PRIORITY,
  squares: controlTints(findings),
});

export const kingSafetyMask = (
  safety: KingSafety,
  pieces: PlacedPiece[],
): OverlayMask => {
  const king = pieces.find(
    (p) => p.kind === "king" && p.color === safety.color,
  );
  return {
    id: maskIdFor.king(safety.color),
    priority: KING_SAFETY_PRIORITY,
    squares: marksOf(
      [...(king ? [king.id] : []), ...safety.attacking_piece_ids],
      pieces,
      "warning",
    ),
    // One arrow per attacker, all converging on the king — which is what
    // "attack_penalty 14" means, drawn.
    arrows: king
      ? safety.attacking_piece_ids.flatMap((id) =>
          arrowBetween(id, king.id, pieces, "warning"),
        )
      : [],
  };
};

export const pawnFlagMask = (
  structure: PawnStructure,
  flag: PawnFlagKey,
  pieces: PlacedPiece[],
): OverlayMask => ({
  id: maskIdFor.pawns(structure.color, flag),
  priority: PAWN_PRIORITY,
  // A passed pawn is a strength; every other flag is a weakness.
  squares: marksOf(
    structure[flag],
    pieces,
    flag === "passed_pawn_ids" ? "good" : "warning",
  ),
});

export const pinMask = (pin: Pin, pieces: PlacedPiece[]): OverlayMask => ({
  id: maskIdFor.pin(pin),
  priority: TACTIC_PRIORITY,
  // Only the three pieces, not `pin.squares` — the ray's empty squares
  // are what the arrow already traces, and lighting both says the same
  // thing twice while burying the pieces that matter.
  squares: marksOf(
    [pin.pinner_piece_id, pin.pinned_piece_id, pin.pin_target_id],
    pieces,
    "danger",
  ),
  // Pinner to target, the line the pinned piece can't step off — the
  // same geometry the panel row draws.
  arrows: arrowBetween(
    pin.pinner_piece_id,
    pin.pin_target_id,
    pieces,
    "danger",
  ),
});

export const forkMask = (fork: Fork, pieces: PlacedPiece[]): OverlayMask => ({
  id: maskIdFor.fork(fork),
  priority: TACTIC_PRIORITY,
  squares: marksOf([fork.forker_id, ...fork.forked_ids], pieces, "danger"),
  arrows: fork.forked_ids.flatMap((victimId) =>
    arrowBetween(fork.forker_id, victimId, pieces, "danger"),
  ),
});

/**
 * One of the engine's moves as a mask — the best move or the main threat,
 * the two rows the Engine tab can put on the board.
 *
 * They're masks for the same reason the findings are: so there's one
 * model of what's on the board. As separate on/off switches they'd keep
 * the old `Overlay` type alive beside the mask ids — their own toggle,
 * their own preview handling, their own case in the clear button — which
 * is the two-systems split this file replaces. As masks, the Engine tab's
 * rows switch them by `maskIdFor` like every finding row, and the arrow
 * takes its paint order from `ENGINE_MOVE_PRIORITY` like everything else.
 *
 * Rebuilt every render like the rest, so a switched-on best move follows
 * the search as it deepens instead of freezing at whatever move it showed
 * when you clicked.
 *
 * No squares: the arrow already starts and ends on the only two that
 * matter. A move that hasn't arrived yet, or won't parse, draws nothing
 * rather than an arrow to the wrong corner.
 */
const engineMoveMask = (
  id: string,
  uci: string | null,
  tone: HighlightTone,
): OverlayMask => {
  const move = uci ? parseUciMove(uci) : null;
  return {
    id,
    priority: ENGINE_MOVE_PRIORITY,
    arrows: move
      ? [
          {
            from: [move.from.rank, move.from.file],
            to: [move.to.rank, move.to.file],
            color: TONE_ARROW_COLOR[tone],
            type: "engine",
          },
        ]
      : [],
  };
};

// One builder per move rather than exporting `engineMoveMask` itself: each
// fixes its own id and colour, so no call site can pair the best move with
// the threat's id, or draw the threat in the best move's green.
export const bestMoveMask = (uci: string | null): OverlayMask =>
  engineMoveMask(maskIdFor.bestMove, uci, "good");

export const threatMask = (uci: string | null): OverlayMask =>
  engineMoveMask(maskIdFor.threat, uci, "danger");

/**
 * Every mask the viewed position's findings can draw.
 *
 * Rebuilt on every render rather than stored, which is what lets a
 * switched-on id follow the game: on a ply where its finding doesn't
 * exist nothing here carries that id, so nothing is drawn, and on a ply
 * where it does, it's back.
 */
export const positionMasks = (
  findings: PositionFindings | null,
  pieces: PlacedPiece[],
): OverlayMask[] => {
  if (!findings) return [];
  const structures = [
    findings.white_pawn_structure,
    findings.black_pawn_structure,
  ];
  return [
    controlMask(findings),
    kingSafetyMask(findings.white_king_safety, pieces),
    kingSafetyMask(findings.black_king_safety, pieces),
    ...structures.flatMap((structure) =>
      PAWN_FLAG_KEYS.map((flag) => pawnFlagMask(structure, flag, pieces)),
    ),
    ...findings.pins.map((pin) => pinMask(pin, pieces)),
    ...findings.forks.map((fork) => forkMask(fork, pieces)),
  ];
};

/**
 * A mask with its arrows ghosted, for a preview — so "I'm looking at
 * this" stays visibly different from "I switched this on".
 */
export const ghosted = (mask: OverlayMask): OverlayMask => ({
  ...mask,
  arrows: mask.arrows?.map((arrow) => ({ ...arrow, type: "ghost" })),
});
