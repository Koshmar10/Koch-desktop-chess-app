import { describe, expect, it } from "vitest";
import { applyMove } from "../applyMove";
import type { PlacedPiece } from "../types";

const whitePawn: PlacedPiece = {
  id: 1,
  kind: "pawn",
  color: "white",
  square: { rank: 6, file: 4 },
};
const blackPawn: PlacedPiece = {
  id: 2,
  kind: "pawn",
  color: "black",
  square: { rank: 1, file: 4 },
};

const E2 = { rank: 6, file: 4 };
const E4 = { rank: 4, file: 4 };
const E7 = { rank: 1, file: 4 };

describe("applyMove", () => {
  it("moves the piece at `from` to `to`", () => {
    const result = applyMove([whitePawn], E2, E4);

    expect(result).toEqual([{ ...whitePawn, square: E4 }]);
  });

  it("removes any piece already standing on the destination square (capture)", () => {
    const result = applyMove([whitePawn, blackPawn], E2, E7);

    expect(result).toEqual([{ ...whitePawn, square: E7 }]);
  });

  it("leaves pieces not involved in the move untouched", () => {
    const bystander: PlacedPiece = {
      id: 3,
      kind: "knight",
      color: "black",
      square: { rank: 0, file: 1 },
    };

    const result = applyMove([whitePawn, bystander], E2, E4);

    expect(result).toContainEqual(bystander);
  });

  it("is a no-op on pieces when `from` has no piece", () => {
    const result = applyMove([blackPawn], E2, E4);

    expect(result).toEqual([blackPawn]);
  });
});
