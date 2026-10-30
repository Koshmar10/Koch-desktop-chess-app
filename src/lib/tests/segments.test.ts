import { describe, expect, it } from "vitest";
import type { Segment } from "../../api/bindings/Segment";
import { segmentIndexOf, segmentsToMarkdown } from "../segments";

describe("segmentsToMarkdown", () => {
  it("keeps text and swaps each tag for a link to its position", () => {
    const segments: Segment[] = [
      { kind: "Text", text: "Play " },
      { kind: "Move", san: "a6", uci: "a7a6" },
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
