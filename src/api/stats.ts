import { invoke } from "@tauri-apps/api/core"
import { PlayerStats } from "./bindings/PlayerStats"
import { RatingPoint } from "./bindings/RatingPoint"

// Aggregate Home-screen numbers, derived on the backend from games /
// analysis / sessions plus the rating table.
export const getPlayerStats = (): Promise<PlayerStats> =>
  invoke<PlayerStats>("get_player_stats")

// Full rating trajectory, oldest first — for a rating-over-time chart.
export const getRatingHistory = (): Promise<RatingPoint[]> =>
  invoke<RatingPoint[]>("get_rating_history")
