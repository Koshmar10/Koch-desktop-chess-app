import { ChevronLeft, ChevronRight } from "lucide-react";
import TooltipButton from "./TooltipButton";

interface PagerProps {
  // Counted from 0; shown counted from 1.
  pageIndex: number;
  pageCount: number;
  onPageChange: (index: number) => void;
}

/** Previous and next page, with "page / pages" between. Nothing for one page. */
const Pager = ({ pageIndex, pageCount, onPageChange }: PagerProps) => {
  if (pageCount <= 1) return null;
  return (
    <div className="flex items-center justify-center gap-3 py-4">
      <TooltipButton
        icon={ChevronLeft}
        tooltip="Previous page"
        onClick={() => onPageChange(pageIndex - 1)}
        disabled={pageIndex === 0}
      />
      <span className="min-w-16 text-center text-sm font-medium tabular-nums">
        {pageIndex + 1}
        <span className="text-foreground/50"> / {pageCount}</span>
      </span>
      <TooltipButton
        icon={ChevronRight}
        tooltip="Next page"
        onClick={() => onPageChange(pageIndex + 1)}
        disabled={pageIndex >= pageCount - 1}
      />
    </div>
  );
};

export default Pager;
