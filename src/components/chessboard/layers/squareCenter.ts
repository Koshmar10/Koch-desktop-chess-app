export function getSquareCenter(
  square: [number, number],
  isFlipped: boolean,
  squareSize: number,
) {
  const [row, col] = square;

  const xIndex = isFlipped ? 7 - col : col;
  const yIndex = isFlipped ? 7 - row : row;

  return {
    x: xIndex * squareSize + squareSize / 2,
    y: yIndex * squareSize + squareSize / 2,
  };
}
