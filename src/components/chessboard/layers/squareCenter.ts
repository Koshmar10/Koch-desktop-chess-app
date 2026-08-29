import { flipCoords } from "../lib/orientation";

export function getSquareCenter(
  square: [number, number],
  isFlipped: boolean,
  squareSize: number,
) {
  const [row, col] = square;
  const [yIndex, xIndex] = flipCoords(row, col, isFlipped);

  return {
    x: xIndex * squareSize + squareSize / 2,
    y: yIndex * squareSize + squareSize / 2,
  };
}
