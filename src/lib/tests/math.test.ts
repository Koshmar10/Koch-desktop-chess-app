import { describe, expect, it } from "vitest";
import { clamp } from "../math";

describe("clamp", () => {
  it("leaves a value inside the range alone", () => {
    expect(clamp(5, 0, 10)).toBe(5);
  });

  it("holds a value outside the range at the nearer end", () => {
    expect(clamp(-3, 0, 10)).toBe(0);
    expect(clamp(42, 0, 10)).toBe(10);
  });
});
