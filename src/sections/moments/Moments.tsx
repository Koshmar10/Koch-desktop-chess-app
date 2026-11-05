import type { MomentSummary } from "../../api/bindings/MomentSummary";
import { useEffect, useState } from "react";
import { listMoments } from "../../api/moments";
import LoadError from "../../components/LoadError";
import LoadingSpinner from "../../components/LoadingSpinner";
import Chessboard from "../../components/chessboard/Chessboard";
import Squares from "../../components/chessboard/layers/Squares";
import PieceLayer from "../../components/chessboard/layers/PieceLayer";
import {
  MOMENT_KIND_ICON,
  MOMENT_KIND_LABEL,
  MOMENT_KIND_TEXT_CLASS,
  formatPlayedDate,
} from "../../lib/moments";
import OverlayLayer from "../../components/chessboard/layers/OverlayLayer";
import { momentMask } from "./masks";

const MOMENT_BOARD_SIZE_PX = 240;
const CARD_PADDING_PX = 8;
const ICON_SIZE_PX = 14;
const PAGE_CENTER_CLASS = "flex h-full items-center justify-center";

interface MomentCardProps {
  moment: MomentSummary;
}

const MomentCard = ({ moment }: MomentCardProps) => {
  const KindIcon = MOMENT_KIND_ICON[moment.kind];

  return (
    <div className="bg-card rounded-md" style={{ padding: CARD_PADDING_PX }}>
      <Chessboard pieces={moment.pieces} sizePx={MOMENT_BOARD_SIZE_PX}>
        <Squares rankLabels={false} fileLabels={false} />
        <OverlayLayer masks={[momentMask(moment)]} />
        <PieceLayer squareRelativePct={200} />
      </Chessboard>
      <div className="flex flex-row items-center justify-between pt-2">
        <p
          className={`flex items-center gap-1 text-sm font-medium ${MOMENT_KIND_TEXT_CLASS[moment.kind]}`}
        >
          <KindIcon size={ICON_SIZE_PX} />
          {MOMENT_KIND_LABEL[moment.kind]}
        </p>
        <p className="text-xs opacity-60">
          {formatPlayedDate(moment.date_played)}
        </p>
      </div>
    </div>
  );
};

const Moments = () => {
  // Null until the list arrives, so loading is read off the list itself
  // rather than kept as a flag that could disagree with it.
  const [moments, setMoments] = useState<MomentSummary[] | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    listMoments({ limit: 40 })
      .then(setMoments)
      .catch((err) => setError(String(err)));
  }, []);

  if (error !== null) {
    return (
      <div className={PAGE_CENTER_CLASS}>
        <LoadError what="your moments" message={error} />
      </div>
    );
  }
  if (moments === null) {
    return (
      <div className={PAGE_CENTER_CLASS}>
        <LoadingSpinner message="Loading moments…" />
      </div>
    );
  }
  return (
    <div
      className="grid gap-4 justify-between w-full px-2 py-2"
      // As many card-wide columns as fit; the grid wraps the rest.
      style={{
        gridTemplateColumns: `repeat(auto-fill, ${MOMENT_BOARD_SIZE_PX + 2 * CARD_PADDING_PX}px)`,
      }}
    >
      {moments.map((moment) => {
        return <MomentCard key={moment.moment_id} moment={moment} />;
      })}
    </div>
  );
};

export default Moments;
