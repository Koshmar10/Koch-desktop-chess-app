import { describe, expect, it } from "vitest";
import type { PieceView } from "../../api/bindings/PieceView";
import type { Segment } from "../../api/bindings/Segment";
import type { SegmentMove } from "../../api/bindings/SegmentMove";
import {
  movePreview,
  playableMoves,
  segmentIndexOf,
  segmentsToMarkdown,
} from "../segments";

const PAWN: PieceView = {
  id: 1,
  kind: "pawn",
  color: "black",
  square: { rank: 1, file: 0 },
};

const move = (san: string, uci: string | null): SegmentMove => ({
  san,
  uci,
  pieces_before: uci === null ? [] : [PAWN],
});

describe("movePreview", () => {
  it("shows a move on its own board with the line before it as the trail", () => {
    const line = [move("a6", "a7a6"), move("Ba4", "b5a4"), move("Nf6", "g8f6")];
    expect(movePreview(line, 2)).toEqual({
      pieces: [PAWN],
      uci: "g8f6",
      trail: ["a7a6", "b5a4"],
    });
    expect(movePreview(line, 0)?.trail).toEqual([]);
  });

  it("is null for a move that isn't verified", () => {
    expect(movePreview([move("Bxe8", null)], 0)).toBeNull();
  });
});

describe("playableMoves", () => {
  it("plays a line up to its first move that isn't verified", () => {
    const line = [move("a6", "a7a6"), move("Bxe8", null), move("Nf6", "g8f6")];
    expect(playableMoves(line)).toEqual(["a7a6"]);
    expect(playableMoves([move("a6", "a7a6"), move("Ba4", "b5a4")])).toEqual([
      "a7a6",
      "b5a4",
    ]);
    expect(playableMoves([move("Bxe8", null)])).toEqual([]);
  });
});

describe("segmentsToMarkdown", () => {
  it("keeps text and swaps each tag for a link to its position", () => {
    const segments: Segment[] = [
      { kind: "Text", text: "Play " },
      { kind: "Move", ...move("a6", "a7a6") },
      { kind: "Text", text: " or " },
      { kind: "Ply", n: 2, label: "move 2" },
    ];
    expect(segmentsToMarkdown(segments)).toBe(
      "Play [chip](koch-segment:1) or [chip](koch-segment:3)",
    );
  });
});

describe("segmentIndexOf", () => {
  it("reads the position back out of a segment link", () => {
    expect(segmentIndexOf("koch-segment:3")).toBe(3);
  });

  it("is null for ordinary links and for a position that isn't a number", () => {
    expect(segmentIndexOf("https://example.com")).toBeNull();
    expect(segmentIndexOf("koch-segment:abc")).toBeNull();
    expect(segmentIndexOf(undefined)).toBeNull();
  });
});
