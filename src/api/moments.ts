import { invoke } from "@tauri-apps/api/core";
import type { MomentDetail } from "./bindings/MomentDetail";
import type { MomentFilter } from "./bindings/MomentFilter";
import type { MomentSummary } from "./bindings/MomentSummary";

// The moments that fit the filter, most recent game first unless it sorts
// otherwise. An empty filter lists them all.
export const listMoments = (filter: MomentFilter): Promise<MomentSummary[]> =>
  invoke<MomentSummary[]>("list_moments", { filter });

// One moment and the board it is about. Null when no moment has that id: a
// rule change can find a game's moments again without it, so a saved id
// can go stale.
export const getMoment = (momentId: number): Promise<MomentDetail | null> =>
  invoke<MomentDetail | null>("get_moment", { momentId });
