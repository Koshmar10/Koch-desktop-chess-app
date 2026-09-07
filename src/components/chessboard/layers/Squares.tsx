import { useChessboardContext } from "../ChessboardContext";
import { flipCoords } from "../lib/orientation";

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

const LegalMoveDot: React.FC = () => (
  <div className="size-[50%] rounded-full bg-gray-600/70" />
);

const LegalMoveRing: React.FC = () => (
  <div className="size-[50%] rounded-full border-6 border-gray-600/70 box-border" />
);

const Squares: React.FC = () => {
  const {
    boardSize,
    squareSize,
    isFlipped,
    selectedSquare,
    selectSquare,
    legalDestinations,
    pieces,
    movePiece,
    clearSelection,
    lastMove,
  } = useChessboardContext();

  const handleSquareClick = (row: number, col: number) => {
    if (!selectedSquare) return;

    const [rank, file] = flipCoords(row, col, isFlipped);
    if (selectedSquare[0] === rank && selectedSquare[1] === file) {
      clearSelection();
      return;
    }

    // Clicking another piece of your own color should switch the
    // selection to it, not attempt an (illegal) move onto your own piece.
    const clickedPiece = pieces.find(
      (p) => p.square.rank === rank && p.square.file === file,
    );
    const selectedPiece = pieces.find(
      (p) =>
        p.square.rank === selectedSquare[0] &&
        p.square.file === selectedSquare[1],
    );
    if (clickedPiece && selectedPiece?.color === clickedPiece.color) {
      selectSquare(rank, file);
      return;
    }

    movePiece(selectedSquare, [rank, file]);
  };

  // `selectedSquare` is a board (rank/file) coordinate; convert it once to
  // the screen position it should highlight, rather than re-converting each
  // grid cell's row/col back to board space just to compare.
  const selectedScreenSquare = selectedSquare
    ? flipCoords(selectedSquare[0], selectedSquare[1], isFlipped)
    : null;

  const squares = Array.from({ length: boardSize * boardSize }, (_, i) => {
    const row = Math.floor(i / boardSize);
    const col = i % boardSize;
    const dark = (row + col) % 2 === 1;
    const isSelected =
      selectedScreenSquare?.[0] === row && selectedScreenSquare?.[1] === col;

    const [rank, file] = flipCoords(row, col, isFlipped);
    const rankLabel = col === 0 ? 8 - rank : null;
    const fileLabel = row === 7 ? FILES[file] : null;
    const labelColor = dark ? "text-white/60" : "text-black/60";
    const isLegalDestination = legalDestinations.some(
      (sq) => sq.rank === rank && sq.file === file,
    );
    const isLastMoveSquare =
      lastMove !== null &&
      ((lastMove.from.rank === rank && lastMove.from.file === file) ||
        (lastMove.to.rank === rank && lastMove.to.file === file));
    // A legal destination with a piece already on it is a capture — those
    // render as a ring around the piece instead of a dot.
    const isCapture =
      isLegalDestination &&
      pieces.some((p) => p.square.rank === rank && p.square.file === file);

    return (
      <div
        key={`${row}-${col}`}
        onClick={() => handleSquareClick(row, col)}
        className="transition-colors duration-150"
        style={{
          position: "relative",
          width: squareSize,
          height: squareSize,
          backgroundColor:
            isSelected || isLastMoveSquare
              ? SELECTED_SQUARE_COLOR
              : dark
                ? DARK_SQUARE_COLOR
                : LIGHT_SQUARE_COLOR,
        }}
      >
        <Rank rankLabel={rankLabel} labelColor={labelColor} />
        <File fileLabel={fileLabel} labelColor={labelColor} />
        {isLegalDestination && (
          <div className="absolute inset-0 flex items-center justify-center pointer-events-none z-2">
            {isCapture ? <LegalMoveRing /> : <LegalMoveDot />}
          </div>
        )}
      </div>
    );
  });

  return (
    <div className="grid grid-cols-8 grid-rows-8 w-full h-full">{squares}</div>
  );
};

export default Squares;
