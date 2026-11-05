import {
  Crosshair,
  ShieldAlert,
  TrendingDown,
  TrendingUp,
  type LucideIcon,
} from "lucide-react";
import type { MomentKind } from "../api/bindings/MomentKind";
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

export const MOMENT_KIND_TEXT_CLASS: Record<MomentKind, string> = {
  material_chance: "text-amber-400",
  threatened_piece: "text-red-400",
  winning_position: "text-green-400",
  losing_position: "text-red-400",
};

/** The day a game was played: "5 Nov 2026". Empty for a missing date. */
export const formatPlayedDate = (stored: string | null): string =>
  parseStoredDate(stored)?.toLocaleDateString(undefined, {
    day: "numeric",
    month: "short",
    year: "numeric",
  }) ?? "";
