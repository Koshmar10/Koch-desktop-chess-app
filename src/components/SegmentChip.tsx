import { MapPin } from "lucide-react";
import type { Segment } from "../api/bindings/Segment";
import type { SegmentMove } from "../api/bindings/SegmentMove";
import { useChatBoard } from "./chatBoard";

const PLY_ICON_SIZE_PX = 12;

const CHIP_CLASS = "rounded bg-primary/15 px-1 font-mono text-sm";
const REJECTED_MOVE_CLASS = "text-destructive line-through";
const REJECTED_MOVE_TITLE =
  "Not verified: not legal in this position, or the position isn't known";

interface MoveTextProps {
  move: SegmentMove;
  // Called with the move while the pointer is on it, then with null.
  // Left out where the move isn't from the position on the board, so
  // there's nothing to draw and nothing to point at.
  onPreview?: (uci: string | null) => void;
}

// A move that has no UCI is struck through rather than hidden, so you can
// see what the assistant claimed and that it couldn't be backed.
const MoveText = ({ move, onPreview }: MoveTextProps) => {
  if (move.uci === null) {
    return (
      <span className={REJECTED_MOVE_CLASS} title={REJECTED_MOVE_TITLE}>
        {move.san}
      </span>
    );
  }
  if (!onPreview) return <span>{move.san}</span>;
  return (
    <span
      className="cursor-pointer"
      onMouseEnter={() => onPreview(move.uci)}
      onMouseLeave={() => onPreview(null)}
    >
      {move.san}
    </span>
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
  const { viewedPly, onGoToPly, onPreviewMove } = useChatBoard();
  switch (segment.kind) {
    case "Move":
      return (
        <span className={CHIP_CLASS}>
          <MoveText
            move={segment}
            onPreview={messagePly === viewedPly ? onPreviewMove : undefined}
          />
        </span>
      );
    case "Line":
      // Only the first move is played from a position that's on the board;
      // the rest are from positions further along the line.
      return (
        <span className={`${CHIP_CLASS} inline-flex flex-wrap gap-x-1`}>
          {segment.moves.map((move, index) => (
            <MoveText
              key={index}
              move={move}
              onPreview={
                index === 0 && segment.start_ply === viewedPly
                  ? onPreviewMove
                  : undefined
              }
            />
          ))}
        </span>
      );
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
