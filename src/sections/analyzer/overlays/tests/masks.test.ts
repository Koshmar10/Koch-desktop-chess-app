import { describe, expect, it } from "vitest";
import type { Fork } from "../../../../api/bindings/Fork";
import type { Pin } from "../../../../api/bindings/Pin";
import { TONE_SQUARE_COLOR } from "../../../../components/chessboard/lib/highlightTones";
import type { PlacedPiece } from "../../../../components/chessboard/lib/types";
import { FIXTURE_FINDINGS, FIXTURE_PIECES } from "../../tests/fixtures";
import {
  CONTROL_PRIORITY,
  ENGINE_MOVE_PRIORITY,
  KING_SAFETY_PRIORITY,
  PAWN_PRIORITY,
  TACTIC_PRIORITY,
  bestMoveMask,
  forkMask,
  ghosted,
  kingSafetyMask,
  maskIdFor,
  pawnFlagMask,
  pinMask,
  positionMasks,
  threatMask,
} from "../masks";

const piece = (
  id: number,
  kind: PlacedPiece["kind"],
  color: PlacedPiece["color"],
  rank: number,
  file: number,
): PlacedPiece => ({ id, kind, color, square: { rank, file } });

// Bishop b5 pins the d7 knight to the e8 king; knight c7 forks the king
// and the a8 rook.
const PIECES: PlacedPiece[] = [
  piece(4, "king", "black", 0, 4), // e8
  piece(13, "knight", "black", 1, 3), // d7
  piece(17, "rook", "black", 0, 0), // a8
  piece(29, "bishop", "white", 3, 1), // b5
  piece(30, "knight", "white", 1, 2), // c7
  piece(40, "pawn", "white", 4, 4), // e4
];

const PIN: Pin = {
  pinner_piece_id: 29,
  pinned_piece_id: 13,
  pin_target_id: 4,
  squares: [],
};
const FORK: Fork = { forker_id: 30, forked_ids: [4, 17] };

describe("maskIdFor", () => {
  it("names a fork by its victims, whatever order they're listed in", () => {
    expect(maskIdFor.fork({ forker_id: 30, forked_ids: [17, 4] })).toBe(
      maskIdFor.fork(FORK),
    );
  });

  // The same knight forking other pieces later is a different fork, and
  // switching one on mustn't light the other.
  it("tells apart two forks by the same piece", () => {
    expect(maskIdFor.fork({ forker_id: 30, forked_ids: [4, 13] })).not.toBe(
      maskIdFor.fork(FORK),
    );
  });
});

describe("pinMask", () => {
  it("lights the three pieces and draws pinner to target", () => {
    const mask = pinMask(PIN, PIECES);

    expect(mask.squares?.map((mark) => mark.square)).toEqual([
      { rank: 3, file: 1 },
      { rank: 1, file: 3 },
      { rank: 0, file: 4 },
    ]);
    expect(mask.arrows).toEqual([
      expect.objectContaining({ from: [3, 1], to: [0, 4] }),
    ]);
  });

  // Findings and pieces a ply apart: draw what's there, not a mark or an
  // arrow pointing at nothing.
  it("skips pieces the position doesn't hold", () => {
    const mask = pinMask(
      PIN,
      PIECES.filter((p) => p.id !== PIN.pin_target_id),
    );

    expect(mask.squares).toHaveLength(2);
    expect(mask.arrows).toEqual([]);
  });
});

describe("forkMask", () => {
  it("draws one arrow from the forker to each victim", () => {
    const mask = forkMask(FORK, PIECES);

    expect(mask.arrows?.map((arrow) => arrow.from)).toEqual([
      [1, 2],
      [1, 2],
    ]);
    expect(mask.arrows?.map((arrow) => arrow.to)).toEqual([
      [0, 4],
      [0, 0],
    ]);
  });
});

describe("kingSafetyMask", () => {
  it("converges an arrow from every attacker on the king", () => {
    const mask = kingSafetyMask(
      {
        ...FIXTURE_FINDINGS.black_king_safety,
        color: "black",
        attacking_piece_ids: [29, 30],
      },
      PIECES,
    );

    expect(mask.id).toBe(maskIdFor.king("black"));
    expect(mask.arrows?.map((arrow) => arrow.to)).toEqual([
      [0, 4],
      [0, 4],
    ]);
  });
});

describe("pawnFlagMask", () => {
  it("fills a passed pawn as a strength and the other flags as weaknesses", () => {
    const structure = {
      ...FIXTURE_FINDINGS.white_pawn_structure,
      color: "white" as const,
      passed_pawn_ids: [40],
      isolated_pawn_ids: [40],
    };

    const passed = pawnFlagMask(structure, "passed_pawn_ids", PIECES);
    const isolated = pawnFlagMask(structure, "isolated_pawn_ids", PIECES);

    expect(passed.squares?.[0].color).toBe(TONE_SQUARE_COLOR.good);
    expect(isolated.squares?.[0].color).toBe(TONE_SQUARE_COLOR.warning);
  });
});

describe("engine move masks", () => {
  it("draws the move as a single arrow", () => {
    expect(bestMoveMask("e2e4").arrows).toEqual([
      expect.objectContaining({ from: [6, 4], to: [4, 4] }),
    ]);
  });

  // Switched on before the engine has said anything: nothing yet, and
  // the arrow appears once a move arrives.
  it("draws nothing until there's a move", () => {
    expect(threatMask(null).arrows).toEqual([]);
  });
});

describe("ghosted", () => {
  it("ghosts a preview's arrows and leaves its squares alone", () => {
    const mask = pinMask(PIN, PIECES);
    const preview = ghosted(mask);

    expect(preview.arrows?.every((arrow) => arrow.type === "ghost")).toBe(true);
    expect(preview.squares).toEqual(mask.squares);
  });
});

describe("positionMasks", () => {
  // A toggle switches whatever carries its id, and the layer keys its
  // squares and arrows by it — two masks sharing one would light together
  // and collide in React.
  it("gives every mask its own id", () => {
    const ids = positionMasks(FIXTURE_FINDINGS, FIXTURE_PIECES).map(
      (mask) => mask.id,
    );
    expect(new Set(ids).size).toBe(ids.length);
  });

  it("has nothing to draw without findings", () => {
    expect(positionMasks(null, PIECES)).toEqual([]);
  });
});

describe("mask priorities", () => {
  it("paints control, then structure, then tactics, then the engine", () => {
    expect(PAWN_PRIORITY).toBeGreaterThan(CONTROL_PRIORITY);
    expect(KING_SAFETY_PRIORITY).toBeGreaterThan(PAWN_PRIORITY);
    expect(TACTIC_PRIORITY).toBeGreaterThan(KING_SAFETY_PRIORITY);
    expect(ENGINE_MOVE_PRIORITY).toBeGreaterThan(TACTIC_PRIORITY);
  });
});
