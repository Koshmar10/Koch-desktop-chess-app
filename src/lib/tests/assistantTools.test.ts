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

  it("shows which ply's opening is looked up", () => {
    expect(toolCallLabel("lookup_opening", '{"ply":6}')).toBe(
      "Looking up the opening at ply 6",
    );
  });

  it("labels the game review, which takes no arguments", () => {
    expect(toolCallLabel("get_game_review", "{}")).toBe("Reviewing the game");
  });

  it("says which line the engine is asked about, if any", () => {
    expect(
      toolCallLabel(
        "candidate_moves",
        '{"from_ply":43,"moves":["Qg4"],"line_plies":6}',
      ),
    ).toBe("Asking the engine about Qg4");
    expect(
      toolCallLabel(
        "candidate_moves",
        '{"from_ply":43,"moves":[],"line_plies":1}',
      ),
    ).toBe("Asking the engine for the best moves");
  });

  it("labels the player profile", () => {
    expect(toolCallLabel("get_player_profile", "{}")).toBe(
      "Looking at how you usually play",
    );
  });

  it("shows an unlabelled tool as it was called", () => {
    expect(toolCallLabel("some_new_tool", '{"ply":4}')).toBe(
      'some_new_tool({"ply":4})',
    );
  });
});
