import { useEffect, useState } from "react";
import { ArrowRight } from "lucide-react";
import { Link } from "react-router-dom";
import { listMoments } from "../../api/moments";
import type { MomentSummary } from "../../api/bindings/MomentSummary";
import EmptyNote from "../../components/EmptyNote";
import { MS_PER_DAY } from "../../lib/time";
import MomentCard from "../moments/MomentCard";
import HomeBlock from "./HomeBlock";

const WEEK_DAYS = 7;
const HIGHLIGHT_COUNT = 5;
const MOMENT_BOARD_SIZE_PX = 180;

// The backend reads a day as "YYYY-MM-DD".
const isoDay = (date: Date) => date.toISOString().slice(0, 10);

const WeekMoments = () => {
  const [moments, setMoments] = useState<MomentSummary[]>([]);

  useEffect(() => {
    listMoments({
      played_from: isoDay(new Date(Date.now() - WEEK_DAYS * MS_PER_DAY)),
      sort: "costliest",
      limit: HIGHLIGHT_COUNT,
    })
      .then(setMoments)
      .catch(console.error);
  }, []);

  return (
    <HomeBlock
      title="This week's moments"
      subtitle="The turning points from your games over the last seven days."
    >
      {moments.length === 0 ? (
        <EmptyNote>No moments from this week</EmptyNote>
      ) : (
        <div className="flex justify-between gap-4">
          {moments.map((moment) => (
            <MomentCard
              key={moment.moment_id}
              moment={moment}
              boardSizePx={MOMENT_BOARD_SIZE_PX}
            />
          ))}
        </div>
      )}
      <Link
        to="/moments"
        className="flex items-center gap-1 self-start text-xs text-primary hover:underline"
      >
        All moments <ArrowRight size={12} />
      </Link>
    </HomeBlock>
  );
};

export default WeekMoments;
