import type { ReactNode } from "react";
import EyeIndicator from "./EyeIndicator";
import { maskVisibility, useMaskSelection } from "./maskSelection";

const TOGGLE_KEYS = new Set(["Enter", " "]);

// Room for the square names that hang below each tile. They sit outside
// the chips' boxes (see `TileChip`), so the row has to make the space.
const CAPTION_ROOM_CLASS = "pb-4";

// Named widths rather than a free class string, so the gap is the one
// layout choice a row makes — and can't conflict with the row's own.
type RowGap = "normal" | "wide";

const GAP_CLASS: Record<RowGap, string> = {
  normal: "gap-x-2",
  wide: "gap-x-4",
};

/** What every row takes, whichever kind it is. */
interface RowProps {
  // The mask this row switches on and off — one of `maskIdFor`'s ids.
  maskId: string;
  // What that mask shows, for the accessible name: "Show <this> on the
  // board", e.g. "this pin".
  maskLabel: string;
  // Set when the row holds chips captioned with their square name.
  captioned?: boolean;
  gap?: RowGap;
  // Full-width content under the main line, for a row with more to say
  // than fits beside its eye — the king's gauge and penalty causes.
  details?: ReactNode;
  children: ReactNode;
}

// How a kind of row looks at rest. The bottom padding is kept apart
// because `captioned` replaces it, and Tailwind settles two conflicting
// classes by their order in the stylesheet, not in the class string — so
// "the later one wins" isn't something to rely on.
interface RowLook {
  frame: string;
  bottom: string;
}

/**
 * The shared row, which each kind of row below fills in with its look —
 * the way a subclass fills in an abstract base. React does this by
 * composition rather than `extends`: a component that wraps another and
 * fixes some of its props is the subclass.
 *
 * A row is the switch for one mask: rest on it to preview it on the
 * board, click to switch it on and leave it there, click again to switch
 * it off. Clicking an icon on every row to see anything made the panel
 * slow to explore; this keeps a look free and a commitment one click
 * away. Works from the keyboard too — focusing a row previews it, and
 * Enter or Space toggles it — since the row is the only control there is.
 *
 * It also owns the row's anatomy — the eye at the far end, room for
 * captions — so a finding only draws its own contents. Every row type
 * used to rebuild those around itself, and pins and forks took two
 * wrapper components each to do it.
 */
const InteractiveRow = ({
  look: { frame, bottom },
  maskId,
  maskLabel,
  captioned = false,
  gap = "normal",
  details,
  children,
}: RowProps & { look: RowLook }) => {
  const selection = useMaskSelection();
  const visibility = maskVisibility(maskId, selection);
  const isShown = visibility === "shown";

  return (
    <div
      role="button"
      tabIndex={0}
      aria-pressed={isShown}
      aria-label={`${isShown ? "Hide" : "Show"} ${maskLabel} on the board`}
      onClick={() => selection.onToggle(maskId)}
      onKeyDown={(event) => {
        if (!TOGGLE_KEYS.has(event.key)) return;
        event.preventDefault();
        selection.onToggle(maskId);
      }}
      onMouseEnter={() => selection.onPreview(maskId)}
      onMouseLeave={() => selection.onPreview(null)}
      onFocus={() => selection.onPreview(maskId)}
      onBlur={() => selection.onPreview(null)}
      className={`flex cursor-pointer flex-wrap items-center gap-y-1.5 rounded-md outline-none transition-colors duration-100 focus-visible:ring-2 focus-visible:ring-primary/50 ${GAP_CLASS[gap]} ${frame} ${captioned ? CAPTION_ROOM_CLASS : bottom}`}
    >
      {children}
      <EyeIndicator visibility={visibility} />
      {/* A full-width item in a wrapping row can't share a line, so it
          drops below everything before it — which keeps the eye at the
          end of the first line without a header wrapper around it. */}
      {details && (
        <div className="flex basis-full flex-col gap-1.5">{details}</div>
      )}
    </div>
  );
};

const PLAIN_LOOK: RowLook = {
  frame: "-mx-1 px-1 pt-1 hover:bg-foreground/5",
  bottom: "pb-1",
};

const CARD_LOOK: RowLook = {
  frame:
    "border-[1px] border-border/60 bg-card/40 px-2 pt-1.5 hover:bg-card/80",
  bottom: "pb-1.5",
};

/**
 * A line among unlike neighbours, with a faint wash under the pointer so
 * it reads as something you can point at before you've learned that it
 * does anything.
 */
export const PlainRow = (props: RowProps) => (
  <InteractiveRow {...props} look={PLAIN_LOOK} />
);

/**
 * One of a run of like findings — pins, forks — bordered so each reads
 * as its own object.
 */
export const CardRow = (props: RowProps) => (
  <InteractiveRow {...props} look={CARD_LOOK} />
);
