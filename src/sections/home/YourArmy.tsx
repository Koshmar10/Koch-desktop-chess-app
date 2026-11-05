import type { PieceType } from "../../api/bindings/PieceType";
import EmptyNote from "../../components/EmptyNote";
import PieceAvatar from "../../components/chessboard/PieceAvatar";
import HomeBlock from "./HomeBlock";
import { SAMPLE_ARMY } from "./sampleData";
import type { PieceAccuracy } from "./types";

const AVATAR_SIZE_PX = 22;

const PIECE_LABEL: Record<PieceType, string> = {
  king: "King",
  queen: "Queen",
  rook: "Rook",
  bishop: "Bishop",
  knight: "Knight",
  pawn: "Pawn",
};

// Only the extremes get a colour, so the two pieces worth a remark stand
// out from the four that aren't.
const CARELESS_BAR_CLASS = "bg-red-400";
const TRUSTED_BAR_CLASS = "bg-emerald-500";
const BAR_CLASS = "bg-primary";

const GaugeRow = ({
  entry,
  note,
  barClass,
}: {
  entry: PieceAccuracy;
  note?: string;
  barClass: string;
}) => (
  <li className="flex items-center gap-3 border-b-[1px] border-border/40 py-1.5">
    <PieceAvatar color="white" kind={entry.piece} size={AVATAR_SIZE_PX} />
    <div className="flex min-w-0 flex-1 flex-col gap-1">
      <span className="truncate text-xs text-foreground">
        {PIECE_LABEL[entry.piece]}
        {note && <span className="text-foreground/50"> {note}</span>}
      </span>
      <div className="h-1 rounded-full bg-muted/40">
        <div
          className={`h-full rounded-full ${barClass}`}
          style={{ width: `${entry.accuracy}%` }}
        />
      </div>
    </div>
    <span className="w-9 shrink-0 text-right font-mono text-xs text-foreground/70">
      {entry.accuracy}%
    </span>
  </li>
);

const YourArmy = () => {
  const careless = SAMPLE_ARMY.reduce((low, entry) =>
    entry.accuracy < low.accuracy ? entry : low,
  );
  const trusted = SAMPLE_ARMY.reduce((high, entry) =>
    entry.accuracy > high.accuracy ? entry : high,
  );

  return (
    <HomeBlock
      title="Your army"
      subtitle="Accuracy of your moves, by the piece you moved."
    >
      <ul className="flex flex-col">
        {SAMPLE_ARMY.map((entry) => {
          if (entry === careless) {
            return (
              <GaugeRow
                key={entry.piece}
                entry={entry}
                note="most careless"
                barClass={CARELESS_BAR_CLASS}
              />
            );
          }
          if (entry === trusted) {
            return (
              <GaugeRow
                key={entry.piece}
                entry={entry}
                note="most trusted"
                barClass={TRUSTED_BAR_CLASS}
              />
            );
          }
          return (
            <GaugeRow key={entry.piece} entry={entry} barClass={BAR_CLASS} />
          );
        })}
      </ul>
      <EmptyNote>Sample data, not from your games yet.</EmptyNote>
    </HomeBlock>
  );
};

export default YourArmy;
