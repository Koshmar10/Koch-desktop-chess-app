import type { MomentSummary } from "../../api/bindings/MomentSummary";
import { useEffect, useState } from "react";
import { listMoments } from "../../api/moments";
import LoadError from "../../components/LoadError";
import LoadingSpinner from "../../components/LoadingSpinner";
import MomentCard, { CARD_PADDING_PX } from "./MomentCard";

const MOMENT_BOARD_SIZE_PX = 240;
const PAGE_CENTER_CLASS = "flex h-full items-center justify-center";

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
        return (
          <MomentCard
            key={moment.moment_id}
            moment={moment}
            boardSizePx={MOMENT_BOARD_SIZE_PX}
          />
        );
      })}
    </div>
  );
};

export default Moments;
