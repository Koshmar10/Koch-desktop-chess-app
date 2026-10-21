import { describe, expect, it } from "vitest";
import { opponentOf, pieceAt, pieceById } from "../pieces";
import type { PlacedPiece } from "../types";

describe("opponentOf", () => {
  it("is the other side, both ways", () => {
    expect(opponentOf("white")).toBe("black");
    expect(opponentOf("black")).toBe("white");
  });
});

const whiteKing: PlacedPiece = {
  id: 28,
  kind: "king",
  color: "white",
  square: { rank: 7, file: 4 },
};
const blackKnight: PlacedPiece = {
  id: 1,
  kind: "knight",
  color: "black",
  square: { rank: 0, file: 1 },
};
const PIECES = [whiteKing, blackKnight];

describe("pieceAt", () => {
  it("finds the piece standing on a square", () => {
    expect(pieceAt(PIECES, { rank: 0, file: 1 })).toBe(blackKnight);
  });

  it("is undefined for an empty square", () => {
    expect(pieceAt(PIECES, { rank: 4, file: 4 })).toBeUndefined();
  });
});

describe("pieceById", () => {
  it("finds the piece with that id", () => {
    expect(pieceById(PIECES, 28)).toBe(whiteKing);
  });

  it("is undefined for an id the position doesn't hold", () => {
    expect(pieceById(PIECES, 99)).toBeUndefined();
  });
});
