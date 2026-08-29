import type { PlacedPiece } from "./types";

export const applyMove = (
  pieces: PlacedPiece[],
  from: [number, number],
  to: [number, number],
): PlacedPiece[] => {
  const withoutCaptured = pieces.filter(
    (p) => !(p.square.rank === to[0] && p.square.file === to[1]),
  );
  return withoutCaptured.map((p) =>
    p.square.rank === from[0] && p.square.file === from[1]
      ? { ...p, square: { rank: to[0], file: to[1] } }
      : p,
  );
};
