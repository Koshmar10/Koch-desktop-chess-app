import { describe, expect, it } from "vitest";
import { applyMove } from "../applyMove";
import type { PlacedPiece } from "../types";

const whitePawn: PlacedPiece = { kind: "pawn", color: "white", row: 6, col: 4 };
const blackPawn: PlacedPiece = { kind: "pawn", color: "black", row: 1, col: 4 };

describe("applyMove", () => {
  it("moves the piece at `from` to `to`", () => {
    const result = applyMove([whitePawn], [6, 4], [4, 4]);

    expect(result).toEqual([{ ...whitePawn, row: 4, col: 4 }]);
  });

  it("removes any piece already standing on the destination square (capture)", () => {
    const result = applyMove([whitePawn, blackPawn], [6, 4], [1, 4]);

    expect(result).toEqual([{ ...whitePawn, row: 1, col: 4 }]);
  });

  it("leaves pieces not involved in the move untouched", () => {
    const bystander: PlacedPiece = {
      kind: "knight",
      color: "black",
      row: 0,
      col: 1,
    };

    const result = applyMove([whitePawn, bystander], [6, 4], [4, 4]);

    expect(result).toContainEqual(bystander);
  });

  it("is a no-op on pieces when `from` has no piece", () => {
    const result = applyMove([blackPawn], [6, 4], [4, 4]);

    expect(result).toEqual([blackPawn]);
  });
});
