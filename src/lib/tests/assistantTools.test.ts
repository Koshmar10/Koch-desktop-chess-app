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

  it("shows the line being tried", () => {
    expect(
      toolCallLabel("explore_line", '{"from_ply":4,"moves":["Nf3","Nc6"]}'),
    ).toBe("Trying Nf3 Nc6");
  });

  it("shows an unlabelled tool as it was called", () => {
    expect(toolCallLabel("some_new_tool", '{"ply":4}')).toBe(
      'some_new_tool({"ply":4})',
    );
  });
});
