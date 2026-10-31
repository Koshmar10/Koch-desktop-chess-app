import { describe, expect, it } from "vitest";
import { toolCallLabel } from "../assistantTools";

describe("toolCallLabel", () => {
  it("shows the ply as the model sent it, which is the board's number", () => {
    expect(toolCallLabel("get_position", '{"ply":12}')).toBe(
      "Looking at ply 12",
    );
  });

  it("still reads when the arguments don't parse", () => {
    expect(toolCallLabel("get_position", "not json")).toBe(
      "Looking at a position",
    );
  });

  it("shows an unlabelled tool as it was called", () => {
    expect(toolCallLabel("explore_line", '{"moves":["e4"]}')).toBe(
      'explore_line({"moves":["e4"]})',
    );
  });
});
