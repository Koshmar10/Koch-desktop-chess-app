import { describe, expect, it, vi } from "vitest";
import type { MaskSelection } from "../../types";
import { maskVisibility } from "../maskSelection";

const PIN = "pin:29-13";
const FORK = "fork:30:4,17";

const selectionWith = ({
  shown = [],
  previewId = null,
}: {
  shown?: string[];
  previewId?: string | null;
}): MaskSelection => ({
  shown: new Set(shown),
  previewId,
  onToggle: vi.fn(),
  onPreview: vi.fn(),
});

describe("maskVisibility", () => {
  it("is hidden when neither switched on nor previewed", () => {
    expect(maskVisibility(PIN, selectionWith({}))).toBe("hidden");
  });

  it("is previewing while the pointer rests on its row", () => {
    expect(maskVisibility(PIN, selectionWith({ previewId: PIN }))).toBe(
      "previewing",
    );
  });

  it("is shown once switched on, whether or not it's also hovered", () => {
    expect(maskVisibility(PIN, selectionWith({ shown: [PIN] }))).toBe("shown");
    expect(
      maskVisibility(PIN, selectionWith({ shown: [PIN], previewId: PIN })),
    ).toBe("shown");
  });

  it("isn't previewing because a different row is", () => {
    expect(maskVisibility(PIN, selectionWith({ previewId: FORK }))).toBe(
      "hidden",
    );
  });
});
