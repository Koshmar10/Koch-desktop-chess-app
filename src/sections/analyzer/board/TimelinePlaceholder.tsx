const TIMELINE_HEIGHT_PX = 132;

/**
 * Reserved space for the timeline strip — the eval curve from
 * `centipawn_history`, a move-quality dot per ply, and the move list, all
 * sharing one ply axis with the scrubber.
 *
 * Empty on purpose: it runs entirely on `GameAnalysis`, which already
 * exists, so it needs no engine session — but it does need a loaded game,
 * and there's no way to load one into this screen yet.
 */
const TimelinePlaceholder = () => (
  <div
    className="flex shrink-0 items-center justify-center rounded-lg border-[1px] border-dashed border-border bg-secondary/20"
    style={{ height: TIMELINE_HEIGHT_PX }}
  >
    <span className="text-xs font-semibold tracking-wide text-foreground/30 uppercase">
      Timeline
    </span>
  </div>
);

export default TimelinePlaceholder;
