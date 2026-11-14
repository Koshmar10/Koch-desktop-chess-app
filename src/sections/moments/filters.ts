import type { MomentFilter } from "../../api/bindings/MomentFilter";
import type { MomentKind } from "../../api/bindings/MomentKind";
import type { MomentOutcome } from "../../api/bindings/MomentOutcome";
import type { MomentSort } from "../../api/bindings/MomentSort";
import type { Phase } from "../../api/bindings/Phase";
import type { PieceType } from "../../api/bindings/PieceType";
import { periodStart, type Period } from "../../lib/period";

/** What the Moments filter bar is set to; `null` is "any". */
export interface MomentFilters {
  handled: boolean | null;
  kind: MomentKind | null;
  outcome: MomentOutcome | null;
  phase: Phase | null;
  piece: PieceType | null;
  period: Period;
  sort: MomentSort;
}

// The moments worth a look first: the ones not handled well, the costliest
// on top. Handled ones are a filter away.
export const DEFAULT_MOMENT_FILTERS: MomentFilters = {
  handled: false,
  kind: null,
  outcome: null,
  phase: null,
  piece: null,
  period: "All Time",
  sort: "costliest",
};

// The outcomes each kind can end in, so the outcome menu offers only those.
export const KIND_OUTCOMES: Record<MomentKind, MomentOutcome[]> = {
  material_chance: ["taken", "missed"],
  threatened_piece: ["saved", "countered", "ignored"],
  winning_position: ["won", "drew", "lost"],
  losing_position: ["won", "drew", "lost"],
};

export const ALL_OUTCOMES: MomentOutcome[] = [
  "taken",
  "missed",
  "saved",
  "countered",
  "ignored",
  "won",
  "drew",
  "lost",
];

export const sameFilters = (a: MomentFilters, b: MomentFilters): boolean =>
  (Object.keys(a) as (keyof MomentFilters)[]).every((key) => a[key] === b[key]);

/** The bar's settings and the typed search as the backend reads them. */
export const toMomentFilter = (
  filters: MomentFilters,
  search: string,
): MomentFilter => ({
  handled: filters.handled ?? undefined,
  kind: filters.kind ?? undefined,
  outcome: filters.outcome ?? undefined,
  phase: filters.phase ?? undefined,
  piece: filters.piece ?? undefined,
  played_from: periodStart(filters.period),
  sort: filters.sort,
  search: search || undefined,
});
