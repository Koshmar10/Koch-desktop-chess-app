import { describe, expect, it } from "vitest";
import { getTakenPieces } from "../takenPieces";
import { STARTING_POSITION } from "../startingPosition";

describe("getTakenPieces", () => {
  it("is empty for the starting position", () => {
    expect(getTakenPieces(STARTING_POSITION, "white")).toEqual([]);
    expect(getTakenPieces(STARTING_POSITION, "black")).toEqual([]);
  });

  it("reports an opponent piece missing from the board as taken", () => {
    const withoutBlackKnight = STARTING_POSITION.filter(
      (p) =>
        !(p.color === "black" && p.kind === "knight" && p.square.file === 1),
    );

    expect(getTakenPieces(withoutBlackKnight, "white")).toEqual([
      ["knight", "black"],
    ]);
    // White hasn't lost anything, so Black has nothing to show as taken.
    expect(getTakenPieces(withoutBlackKnight, "black")).toEqual([]);
  });

  it("reports multiple missing pieces of the same kind", () => {
    const withoutBothBlackKnights = STARTING_POSITION.filter(
      (p) => !(p.color === "black" && p.kind === "knight"),
    );

    expect(getTakenPieces(withoutBothBlackKnights, "white")).toEqual([
      ["knight", "black"],
      ["knight", "black"],
    ]);
  });
});
