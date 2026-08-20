export interface PieceVisualState {
  className: string;
  style: React.CSSProperties;
}

export const getPieceVisualState = (
  isBeingDragged: boolean,
  dragPosition: { x: number; y: number } | null,
  row: number,
  col: number,
  squareSize: number,
): PieceVisualState => {
  if (isBeingDragged && dragPosition) {
    return {
      className:
        "fixed p-2 box-border select-none pointer-events-none z-50 transition-[width,height] duration-100 ease-out",
      style: {
        left: dragPosition.x - squareSize / 2,
        top: dragPosition.y - squareSize / 2,
        width: squareSize * 1.1,
        height: squareSize * 1.1,
      },
    };
  }

  return {
    className: "p-2 box-border select-none cursor-grab",
    style: {
      gridRowStart: row + 1,
      gridColumnStart: col + 1,
      width: squareSize,
      height: squareSize,
    },
  };
};
