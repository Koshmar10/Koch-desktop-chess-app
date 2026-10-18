import { ChessKnight, Clock, Flame, Zap, type LucideIcon } from "lucide-react";
import type { TimeControl } from "../api/bindings/TimeControl";
import { MS_PER_MINUTE } from "./time";

/** A preset clock. The backend never sees the name, only its `TimeControl`. */
export type GameMode = "Bullet" | "Blitz" | "Rapid" | "Classical";

/** Fastest first — the order the menu lists them in. */
export const GAME_MODES: readonly GameMode[] = [
  "Bullet",
  "Blitz",
  "Rapid",
  "Classical",
];

export const MODE_TIME_CONTROL: Record<GameMode, TimeControl> = {
  Bullet: { initial_ms: 60_000, increment_ms: 0 },
  Blitz: { initial_ms: 180_000, increment_ms: 0 },
  Rapid: { initial_ms: 600_000, increment_ms: 0 },
  Classical: { initial_ms: 1_800_000, increment_ms: 0 },
};

/**
 * One icon per mode, wherever a mode is shown — the play controls and the
 * history cards each used to pick their own, and had to be kept matching
 * by hand.
 */
export const MODE_ICON: Record<GameMode, LucideIcon> = {
  Bullet: Flame,
  Blitz: Zap,
  Rapid: Clock,
  Classical: ChessKnight,
};

/** A mode's clock the way its menu row says it: "10 min". */
export const modeMinutesLabel = (mode: GameMode): string =>
  `${MODE_TIME_CONTROL[mode].initial_ms / MS_PER_MINUTE} min`;

/**
 * Which mode a saved game's clock falls in: the fastest one it's no
 * longer than. Anything longer than every preset counts as Classical.
 */
export const modeForTimeControl = ({ initial_ms }: TimeControl): GameMode =>
  GAME_MODES.find((mode) => initial_ms <= MODE_TIME_CONTROL[mode].initial_ms) ??
  "Classical";
