import { describe, expect, it, vi } from "vitest";
import type { MaskSelection } from "../../types";
import { maskControls } from "../rowControls";

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

const stateOf = (id: string, selection: MaskSelection) =>
  maskControls(id, "this pin", selection).state;

describe("maskControls", () => {
  it("is hidden when neither switched on nor previewed", () => {
    expect(stateOf(PIN, selectionWith({}))).toBe("hidden");
  });

  it("is previewing while the pointer rests on it", () => {
    expect(stateOf(PIN, selectionWith({ previewId: PIN }))).toBe("previewing");
  });

  it("is shown once switched on, whether or not it's also hovered", () => {
    expect(stateOf(PIN, selectionWith({ shown: [PIN] }))).toBe("shown");
    expect(stateOf(PIN, selectionWith({ shown: [PIN], previewId: PIN }))).toBe(
      "shown",
    );
  });

  it("isn't previewing because a different row is", () => {
    expect(stateOf(PIN, selectionWith({ previewId: FORK }))).toBe("hidden");
  });

  it("previews itself on enter and clears the preview on leave", () => {
    const selection = selectionWith({});
    const controls = maskControls(PIN, "this pin", selection);

    controls.onPreview(true);
    controls.onPreview(false);

    expect(selection.onPreview).toHaveBeenNthCalledWith(1, PIN);
    expect(selection.onPreview).toHaveBeenNthCalledWith(2, null);
  });

  it("toggles the mask it was built for", () => {
    const selection = selectionWith({});
    maskControls(PIN, "this pin", selection).onToggle();

    expect(selection.onToggle).toHaveBeenCalledWith(PIN);
  });
});
