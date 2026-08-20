import { useChessboardContext } from "../ChessboardContext";

const FILES = ["a", "b", "c", "d", "e", "f", "g", "h"];
const DARK_SQUARE_COLOR = "#a37a58";
const LIGHT_SQUARE_COLOR = "#f0d9b5";
const SELECTED_SQUARE_COLOR = "#f6f669";

interface RankProps {
  rankLabel: number | null;
  labelColor: string;
}

const Rank: React.FC<RankProps> = ({ rankLabel, labelColor }: RankProps) => {
  if (!rankLabel) {
    return;
  }
  return (
    <span
      className={`absolute top-0.5 left-1 text-[14px] font-bold select-none pointer-events-none ${labelColor}`}
    >
      {rankLabel}
    </span>
  );
};

interface FileProps {
  fileLabel: string | null;
  labelColor: string;
}

const File: React.FC<FileProps> = ({ fileLabel, labelColor }: FileProps) => {
  if (!fileLabel) {
    return;
  }
  return (
    <span
      className={`absolute bottom-0 right-1 text-[14px] font-bold select-none pointer-events-none ${labelColor}`}
    >
      {fileLabel}
    </span>
  );
};

const Squares: React.FC = () => {
  const { boardSize, squareSize, selectedSquare, movePiece, clearSelection } =
    useChessboardContext();

  const handleSquareClick = (row: number, col: number) => {
    if (!selectedSquare) return;

    if (selectedSquare[0] === row && selectedSquare[1] === col) {
      clearSelection();
    } else {
      movePiece(selectedSquare, [row, col]);
    }
  };

  const squares = Array.from({ length: boardSize * boardSize }, (_, i) => {
    const row = Math.floor(i / boardSize);
    const col = i % boardSize;
    const dark = (row + col) % 2 === 1;
    const isSelected =
      selectedSquare?.[0] === row && selectedSquare?.[1] === col;

    const rankLabel = col === 0 ? 8 - row : null;
    const fileLabel = row === 7 ? FILES[col] : null;
    const labelColor = dark ? "text-white/60" : "text-black/60";

    return (
      <div
        key={`${row}-${col}`}
        onClick={() => handleSquareClick(row, col)}
        style={{
          position: "relative",
          width: squareSize,
          height: squareSize,
          backgroundColor: isSelected
            ? SELECTED_SQUARE_COLOR
            : dark
              ? DARK_SQUARE_COLOR
              : LIGHT_SQUARE_COLOR,
        }}
      >
        <Rank rankLabel={rankLabel} labelColor={labelColor} />
        <File fileLabel={fileLabel} labelColor={labelColor} />
      </div>
    );
  });

  return (
    <div className="grid grid-cols-8 grid-rows-8 w-full h-full">{squares}</div>
  );
};

export default Squares;
