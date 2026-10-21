import { useEffect, useRef } from "react";
import EmptyNote from "../../components/EmptyNote";
import PieceAvatar from "../../components/chessboard/PieceAvatar";
import { squareName } from "../../components/chessboard/lib/squareName";
import type { Mistake } from "./types";

const PIECE_SIZE_PX = 20;

interface MistakeListProps {
  mistakes: Mistake[];
}

const MistakeList = ({ mistakes }: MistakeListProps) => {
  const listRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const list = listRef.current;
    if (list) list.scrollLeft = list.scrollWidth;
  }, [mistakes.length]);

  if (mistakes.length === 0) return <EmptyNote>No mistakes yet</EmptyNote>;

  return (
    <div ref={listRef} className="flex gap-1.5 overflow-x-auto pb-1">
      {mistakes.map(({ piece, move, isCapture }, index) => (
        <span
          // Only ever appended to, so the index is stable.
          key={index}
          title={`${piece.kind} ${squareName(move.from)} → ${squareName(move.to)}`}
          className="flex shrink-0 items-center gap-1 rounded-md border-[1px] border-destructive/40 bg-destructive/10 py-0.5 pr-2 pl-0.5 text-xs font-medium text-foreground/80"
        >
          <PieceAvatar
            color={piece.color}
            kind={piece.kind}
            size={PIECE_SIZE_PX}
          />
          {isCapture && "x"}
          {squareName(move.to)}
        </span>
      ))}
    </div>
  );
};

export default MistakeList;
