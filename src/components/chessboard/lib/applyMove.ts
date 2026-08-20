import type { PlacedPiece } from "./types";

export const applyMove = (
  pieces: PlacedPiece[],
  from: [number, number],
  to: [number, number],
): PlacedPiece[] => {
  const withoutCaptured = pieces.filter(
    (p) => !(p.row === to[0] && p.col === to[1]),
  );
  return withoutCaptured.map((p) =>
    p.row === from[0] && p.col === from[1]
      ? { ...p, row: to[0], col: to[1] }
      : p,
  );
};
