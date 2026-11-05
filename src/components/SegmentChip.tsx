import { MapPin } from "lucide-react";
import type { Segment } from "../api/bindings/Segment";
import { useChatBoard } from "./chatBoard";
import MovesChip from "./MovesChip";

const PLY_ICON_SIZE_PX = 12;

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
