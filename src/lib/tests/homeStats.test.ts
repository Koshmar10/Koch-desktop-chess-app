import { describe, expect, it } from "vitest";
import type { GameSummary } from "../../api/bindings/GameSummary";
import type { PieceColor } from "../../api/bindings/PieceColor";
import { openingShares } from "../homeStats";

const game = (opening: string | null, color: PieceColor | null) =>
  ({ opening_name: opening, human_color: color }) as GameSummary;

describe("openingShares", () => {
  it("returns nothing when the colour has no named openings", () => {
    expect(
      openingShares([game(null, "white"), game("A", "black")], "white"),
    ).toEqual([]);
  });

  it("ranks openings and folds the tail into Other", () => {
    const games = [
      ...Array(4).fill(game("A", "white")),
      ...Array(3).fill(game("B", "white")),
      ...Array(2).fill(game("C", "white")),
      game("D", "white"),
    ];

    expect(openingShares(games, "white")).toEqual([
      { name: "A", count: 4, percent: 40 },
      { name: "B", count: 3, percent: 30 },
      { name: "C", count: 2, percent: 20 },
      { name: "Other", count: 1, percent: 10 },
    ]);
  });
});
