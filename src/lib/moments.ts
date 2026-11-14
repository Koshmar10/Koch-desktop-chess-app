import {
  Crosshair,
  ShieldAlert,
  TrendingDown,
  TrendingUp,
  type LucideIcon,
} from "lucide-react";
import type { MomentKind } from "../api/bindings/MomentKind";
import type { MomentOutcome } from "../api/bindings/MomentOutcome";
import { parseStoredDate } from "./time";

export const MOMENT_KIND_LABEL: Record<MomentKind, string> = {
  material_chance: "Material chance",
  threatened_piece: "Threatened piece",
  winning_position: "Winning position",
  losing_position: "Losing position",
};

export const MOMENT_KIND_ICON: Record<MomentKind, LucideIcon> = {
  material_chance: Crosshair,
  threatened_piece: ShieldAlert,
  winning_position: TrendingUp,
  losing_position: TrendingDown,
};

export const MOMENT_OUTCOME_LABEL: Record<MomentOutcome, string> = {
  taken: "Taken",
  missed: "Missed",
  saved: "Saved",
  countered: "Countered",
  ignored: "Ignored",
  won: "Won",
  drew: "Drew",
  lost: "Lost",
};

// The dot beside a moment's outcome: green when the player handled it
// well, red when they didn't. The word beside it stays neutral.
export const HANDLED_DOT_CLASS = "bg-emerald-500";
export const UNHANDLED_DOT_CLASS = "bg-red-400";

/** The day a game was played: "5 Nov 2026". Empty for a missing date. */
export const formatPlayedDate = (stored: string | null): string =>
  parseStoredDate(stored)?.toLocaleDateString(undefined, {
    day: "numeric",
    month: "short",
    year: "numeric",
  }) ?? "";
