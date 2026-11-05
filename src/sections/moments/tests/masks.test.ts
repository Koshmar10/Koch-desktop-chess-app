import { describe, expect, it } from "vitest";
import type { MomentSummary } from "../../../api/bindings/MomentSummary";
import { TONE_SQUARE_COLOR } from "../../../components/chessboard/lib/highlightTones";
import { momentMask } from "../masks";

const E5 = { rank: 3, file: 4 };

const MOMENT: MomentSummary = {
  moment_id: 7,
  game_id: 1,
  start_ply: 5,
  ply_count: 1,
  pieces: [],
  kind: "material_chance",
  outcome: "missed",
  phase: "opening",
  piece: "pawn",
  square: E5,
  eval_before_cp: 150,
  eval_after_cp: 20,
  human_color: "white",
  opponent_name: "Bo",
  date_played: null,
  human_score: "Win",
  opening_name: null,
};

describe("momentMask", () => {
  it("fills the moment's square in its kind's tone", () => {
    expect(momentMask(MOMENT).squares).toEqual([
      { square: E5, color: TONE_SQUARE_COLOR.warning },
    ]);
    expect(momentMask({ ...MOMENT, kind: "threatened_piece" }).squares).toEqual(
      [{ square: E5, color: TONE_SQUARE_COLOR.danger }],
    );
  });

  it("marks nothing for a position, which is about no one square", () => {
    const position: MomentSummary = {
      ...MOMENT,
      kind: "winning_position",
      piece: null,
      square: null,
    };

    expect(momentMask(position).squares).toEqual([]);
  });

  it("has an id per moment, so two cards never share a key", () => {
    expect(momentMask(MOMENT).id).not.toBe(
      momentMask({ ...MOMENT, moment_id: 8 }).id,
    );
  });
});
