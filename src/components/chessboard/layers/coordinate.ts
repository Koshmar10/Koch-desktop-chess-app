// Raw pixel position (MouseEvent.clientX/clientY) — distinct from the
// board's [row, col]/[rank, file] tuples (see `lib/orientation.ts`), which
// are domain/screen grid coordinates, not screen pixels. Local to
// PieceLayer: nothing else in the chessboard subsystem tracks raw cursor
// position.
export interface Coordinate {
  x: number;
  y: number;
}

export const coordinateFromMouseEvent = (
  e: MouseEvent | React.MouseEvent,
): Coordinate => ({ x: e.clientX, y: e.clientY });

export const coordinatesEqual = (a: Coordinate, b: Coordinate): boolean =>
  a.x === b.x && a.y === b.y;
