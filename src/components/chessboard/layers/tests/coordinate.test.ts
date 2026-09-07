import { describe, expect, it } from "vitest";
import { coordinateFromMouseEvent, coordinatesEqual } from "../coordinate";

describe("coordinateFromMouseEvent", () => {
  it("reads clientX/clientY off the event", () => {
    const event = { clientX: 12, clientY: 34 } as MouseEvent;

    expect(coordinateFromMouseEvent(event)).toEqual({ x: 12, y: 34 });
  });
});

describe("coordinatesEqual", () => {
  it("is true for coordinates with equal values", () => {
    expect(coordinatesEqual({ x: 1, y: 2 }, { x: 1, y: 2 })).toBe(true);
  });

  it("is false for coordinates that differ in either axis", () => {
    expect(coordinatesEqual({ x: 1, y: 2 }, { x: 1, y: 3 })).toBe(false);
    expect(coordinatesEqual({ x: 1, y: 2 }, { x: 2, y: 2 })).toBe(false);
  });
});
