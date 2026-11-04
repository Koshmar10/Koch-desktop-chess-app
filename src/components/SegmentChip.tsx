import { MapPin } from "lucide-react";
import type { Segment } from "../api/bindings/Segment";
import type { SegmentMove } from "../api/bindings/SegmentMove";
import { movePreview, type MovePreview } from "../lib/segments";
import { useChatBoard } from "./chatBoard";

const PLY_ICON_SIZE_PX = 12;

// Its own text colour, so a chip stays readable in a dimmed answer.
const CHIP_CLASS =
  "inline-flex flex-wrap gap-x-1 rounded bg-primary/15 px-1 font-mono text-sm text-foreground";
// A chip whose moves start elsewhere: clicking it moves the board there.
const GO_TO_CHIP_CLASS =
  "cursor-pointer underline decoration-dotted underline-offset-2 hover:bg-primary/30";
const PREVIEWED_MOVE_CLASS = "cursor-pointer rounded hover:bg-primary/40";
const REJECTED_MOVE_CLASS = "text-destructive line-through";
const REJECTED_MOVE_TITLE =
  "Not verified: not legal in this position, or the position isn't known";

interface MoveTextProps {
  move: SegmentMove;
  // The move on its own board; null for one that isn't verified.
  preview: MovePreview | null;
}

// A move that has no UCI is struck through rather than hidden, so you can
// see what the assistant claimed and that it couldn't be backed. One that
// has shows itself on the board when the pointer reaches it; the chip,
// not the move, takes it off again (see `MovesChip`).
const MoveText = ({ move, preview }: MoveTextProps) => {
  const { onPreviewMove } = useChatBoard();
  if (preview === null) {
    return (
      <span className={REJECTED_MOVE_CLASS} title={REJECTED_MOVE_TITLE}>
        {move.san}
      </span>
    );
  }
  return (
    <span
      className={PREVIEWED_MOVE_CLASS}
      onMouseEnter={() => onPreviewMove(preview)}
    >
      {move.san}
    </span>
  );
};

interface MovesChipProps {
  moves: SegmentMove[];
  // The ply whose position the first move is played from.
  startPly: number;
}

// One move or a line. Each move carries its own board, so hovering it
// shows it wherever the board is, and running the pointer along a line
// plays it out. Off the line's start, clicking takes the board there.
//
// The preview ends when the pointer leaves the chip, not a move: in the
// gap between two moves the last one stays on the board, instead of the
// board snapping back to the game and forward again.
const MovesChip = ({ moves, startPly }: MovesChipProps) => {
  const { viewedPly, onGoToPly, onPreviewMove } = useChatBoard();
  const isOnBoard = startPly === viewedPly;
  const moveTexts = moves.map((move, index) => (
    <MoveText key={index} move={move} preview={movePreview(moves, index)} />
  ));
  const endPreview = () => onPreviewMove(null);
  if (isOnBoard) {
    return (
      <span className={CHIP_CLASS} onMouseLeave={endPreview}>
        {moveTexts}
      </span>
    );
  }
  return (
    <button
      type="button"
      title={`Go to ply ${startPly + 1}, where this is played from`}
      onClick={() => onGoToPly(startPly)}
      onMouseLeave={endPreview}
      className={`${CHIP_CLASS} ${GO_TO_CHIP_CLASS}`}
    >
      {moveTexts}
    </button>
  );
};

interface SegmentChipProps {
  // A move tag; plain text is rendered by the markdown around it.
  segment: Exclude<Segment, { kind: "Text" }>;
  // The ply the answer was asked at: what a single move is played from.
  messagePly: number;
}

// What a `<move>`, `<line>` or `<ply>` tag in an answer looks like inline.
const SegmentChip = ({ segment, messagePly }: SegmentChipProps) => {
  const { onGoToPly } = useChatBoard();
  switch (segment.kind) {
    case "Move":
      return <MovesChip moves={[segment]} startPly={messagePly} />;
    case "Line":
      return <MovesChip moves={segment.moves} startPly={segment.start_ply} />;
    case "Ply":
      return (
        <button
          type="button"
          title={`Go to ply ${segment.n + 1}`}
          onClick={() => onGoToPly(segment.n)}
          className="inline-flex cursor-pointer items-center gap-0.5 text-primary underline-offset-2 hover:underline"
        >
          <MapPin size={PLY_ICON_SIZE_PX} />
          {segment.label}
        </button>
      );
  }
};

export default SegmentChip;
