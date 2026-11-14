import type { MomentFilter } from "../../api/bindings/MomentFilter";
import type { MomentSummary } from "../../api/bindings/MomentSummary";
import { Eye, HistoryIcon, RotateCcw, Sparkles } from "lucide-react";
import { useEffect, useMemo, useRef, useState } from "react";
import { useNavigate } from "react-router-dom";
import { countMoments, listMoments } from "../../api/moments";
import { OutlineButton, PrimaryButton } from "../../components/buttons";
import EmptyNote from "../../components/EmptyNote";
import EmptyState from "../../components/EmptyState";
import LoadError from "../../components/LoadError";
import LoadingSpinner from "../../components/LoadingSpinner";
import Pager from "../../components/Pager";
import {
  DEFAULT_MOMENT_FILTERS,
  sameFilters,
  toMomentFilter,
  type MomentFilters,
} from "./filters";
import MomentCard, { CARD_PADDING_PX } from "./MomentCard";
import MomentFilterBar from "./MomentFilterBar";
import MomentOverlay from "./MomentOverlay";

const MOMENT_BOARD_SIZE_PX = 240;
const PAGE_SIZE = 24;
const PAGE_CENTER_CLASS = "flex h-full items-center justify-center";
// Long enough that typing a name searches once, not once per letter.
const SEARCH_DEBOUNCE_MS = 300;
const ACTION_ICON_SIZE_PX = 16;

interface MomentPage {
  // Which request it answers, so a page for an older filter reads as loading.
  query: MomentFilter;
  index: number;
  moments: MomentSummary[];
  pageCount: number;
  // Whether there are moments at all, whatever the filter.
  anyAtAll: boolean;
}

interface FilteredOutProps {
  filters: MomentFilters;
  search: string;
  onChange: (filters: MomentFilters) => void;
  onSearchChange: (search: string) => void;
}

// There are moments, but the filters hide them all. On the defaults that
// means every moment went well, which is worth saying as such.
const FilteredOut = ({
  filters,
  search,
  onChange,
  onSearchChange,
}: FilteredOutProps) => {
  const onDefaults = sameFilters(filters, DEFAULT_MOMENT_FILTERS) && !search;
  return (
    <div className="flex flex-col items-center gap-3">
      <EmptyNote>
        {onDefaults
          ? "Every moment so far was handled well"
          : "No moments match these filters"}
      </EmptyNote>
      {onDefaults ? (
        <OutlineButton onClick={() => onChange({ ...filters, handled: null })}>
          <Eye size={ACTION_ICON_SIZE_PX} /> Show them
        </OutlineButton>
      ) : (
        <OutlineButton
          onClick={() => {
            onChange(DEFAULT_MOMENT_FILTERS);
            onSearchChange("");
          }}
        >
          <RotateCcw size={ACTION_ICON_SIZE_PX} /> Reset filters
        </OutlineButton>
      )}
    </div>
  );
};

const Moments = () => {
  const navigate = useNavigate();
  const [filters, setFilters] = useState<MomentFilters>(DEFAULT_MOMENT_FILTERS);
  const [search, setSearch] = useState("");
  const [searched, setSearched] = useState("");
  const [pageIndex, setPageIndex] = useState(0);
  const [page, setPage] = useState<MomentPage | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [openMoment, setOpenMoment] = useState<MomentSummary | null>(null);
  const top = useRef<HTMLDivElement>(null);

  const query = useMemo(
    () => toMomentFilter(filters, searched),
    [filters, searched],
  );

  useEffect(() => {
    const timer = setTimeout(() => {
      setSearched(search.trim());
      setPageIndex(0);
    }, SEARCH_DEBOUNCE_MS);
    return () => clearTimeout(timer);
  }, [search]);

  useEffect(() => {
    let stale = false;
    // Counted with every page, so moments found by an analysis that finished
    // meanwhile show up in the page count too.
    Promise.all([
      listMoments({
        ...query,
        limit: PAGE_SIZE,
        offset: pageIndex * PAGE_SIZE,
      }),
      countMoments(query),
    ])
      // Counting them all only when the filter found none, to tell an empty
      // collection from one the filter hides.
      .then(async ([moments, total]) => {
        const anyAtAll = total > 0 || (await countMoments({})) > 0;
        if (stale) return;
        setError(null);
        setPage({
          query,
          index: pageIndex,
          moments,
          pageCount: Math.ceil(total / PAGE_SIZE),
          anyAtAll,
        });
      })
      .catch((err) => {
        if (!stale) setError(String(err));
      });
    return () => {
      stale = true;
    };
  }, [query, pageIndex]);

  const changeFilters = (next: MomentFilters) => {
    setFilters(next);
    setPageIndex(0);
  };

  const goToPage = (index: number) => {
    setPageIndex(index);
    top.current?.scrollIntoView();
  };

  const content = () => {
    if (error !== null) {
      return (
        <div className={PAGE_CENTER_CLASS}>
          <LoadError what="your moments" message={error} />
        </div>
      );
    }
    if (page === null || page.query !== query || page.index !== pageIndex) {
      return (
        <div className={PAGE_CENTER_CLASS}>
          <LoadingSpinner message="Loading moments…" />
        </div>
      );
    }
    if (page.moments.length === 0) {
      return (
        <div className={PAGE_CENTER_CLASS}>
          {page.anyAtAll ? (
            <FilteredOut
              filters={filters}
              search={searched}
              onChange={changeFilters}
              onSearchChange={setSearch}
            />
          ) : (
            <EmptyState
              icon={Sparkles}
              message="Moments are the chances and threats found in your analysed games. Analyse a game from History to find yours."
            >
              <PrimaryButton onClick={() => navigate("/history")}>
                <HistoryIcon size={ACTION_ICON_SIZE_PX} /> Go to History
              </PrimaryButton>
            </EmptyState>
          )}
        </div>
      );
    }
    return (
      <>
        <div
          className="grid gap-4 justify-between w-full px-2 py-2"
          // As many card-wide columns as fit; the grid wraps the rest.
          style={{
            gridTemplateColumns: `repeat(auto-fill, ${MOMENT_BOARD_SIZE_PX + 2 * CARD_PADDING_PX}px)`,
          }}
        >
          {page.moments.map((moment) => (
            <MomentCard
              key={moment.moment_id}
              moment={moment}
              boardSizePx={MOMENT_BOARD_SIZE_PX}
              onOpen={() => setOpenMoment(moment)}
            />
          ))}
        </div>
        <Pager
          pageIndex={pageIndex}
          pageCount={page.pageCount}
          onPageChange={goToPage}
        />
      </>
    );
  };

  return (
    // The page scrolls inside its own box, so the overlay can cover exactly
    // this box (not the sidebar) and stay put while the cards scroll under it.
    <div className="relative flex h-full w-full flex-col">
      {openMoment !== null && (
        <MomentOverlay
          key={openMoment.moment_id}
          moment={openMoment}
          onClose={() => setOpenMoment(null)}
        />
      )}
      <MomentFilterBar
        filters={filters}
        onChange={changeFilters}
        search={search}
        onSearchChange={setSearch}
      />
      <div ref={top} className="flex min-h-0 flex-1 flex-col overflow-y-auto">
        {content()}
      </div>
    </div>
  );
};

export default Moments;
