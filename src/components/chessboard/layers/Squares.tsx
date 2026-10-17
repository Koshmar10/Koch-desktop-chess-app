import type { Square } from "../../../api/bindings/Square";
import { useChessboardContext } from "../ChessboardContext";
import { fromScreen } from "../lib/orientation";
import { pieceAt } from "../lib/pieces";
import { isDarkSquare, sameSquare } from "../lib/square";
import { fileLetter, rankNumber } from "../lib/squareName";
import { DARK_SQUARE_COLOR, LIGHT_SQUARE_COLOR } from "../lib/constants";
import { LEGAL_MOVE_MARKER_Z_INDEX } from "../lib/zIndex";

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

  const handleSquareClick = (square: Square) => {
    if (!selectedSquare) return;

    if (sameSquare(selectedSquare, square)) {
      clearSelection();
      return;
    }

    // Clicking another piece of your own color should switch the
    // selection to it, not attempt an (illegal) move onto your own piece.
    const clickedPiece = pieceAt(pieces, square);
    const selectedPiece = pieceAt(pieces, selectedSquare);
    if (clickedPiece && selectedPiece?.color === clickedPiece.color) {
      selectSquare(square);
      return;
    }

    movePiece(selectedSquare, square);
  };

  // Laid out in screen order — the grid fills from the top-left — with each
  // cell then working out which board square it shows.
  const squares = Array.from({ length: boardSize * boardSize }, (_, i) => {
    const row = Math.floor(i / boardSize);
    const col = i % boardSize;
    const square = fromScreen([row, col], isFlipped);
    const dark = isDarkSquare(square);
    const isSelected = sameSquare(selectedSquare, square);

    // Coordinates label the screen's left column and bottom row, whichever
    // side of the board is at the bottom.
    const rankLabel = col === 0 ? rankNumber(square) : null;
    const fileLabel = row === boardSize - 1 ? fileLetter(square) : null;
    const labelColor = dark ? "text-white/60" : "text-black/60";
    const isLegalDestination = legalDestinations.some((destination) =>
      sameSquare(destination, square),
    );
    const isLastMoveSquare =
      lastMove !== null &&
      (sameSquare(lastMove.from, square) || sameSquare(lastMove.to, square));
    // A legal destination with a piece already on it is a capture — those
    // render as a ring around the piece instead of a dot.
    const isCapture =
      isLegalDestination && pieceAt(pieces, square) !== undefined;

    return (
      <div
        key={`${row}-${col}`}
        onClick={() => handleSquareClick(square)}
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
          <div
            className="absolute inset-0 flex items-center justify-center pointer-events-none"
            style={{ zIndex: LEGAL_MOVE_MARKER_Z_INDEX }}
          >
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
