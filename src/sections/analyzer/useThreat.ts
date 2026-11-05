import { useEffect, useState } from "react";
import { findThreat } from "../../api/analyzer";

// How long the board has to stay on a position before its threat is asked
// for. Each search starts a Stockfish of its own, so stepping through a
// game shouldn't start one for every position passed on the way.
const THREAT_DELAY_MS = 300;

interface Threat {
  fen: string;
  uci: string | null;
}

/**
 * The threat in `fen`, as UCI, once the board has settled on it; null
 * while it's being found, when there's none, and with `enabled` off (the
 * engine stopped). Kept with the FEN it's for, so one that comes back late
 * for a position already left is never shown on the new one.
 */
export const useThreat = (fen: string | null, enabled: boolean) => {
  const [threat, setThreat] = useState<Threat | null>(null);

  useEffect(() => {
    if (!enabled || fen === null) return;
    const timer = setTimeout(() => {
      findThreat(fen)
        .then((uci) => setThreat({ fen, uci }))
        .catch(console.error);
    }, THREAT_DELAY_MS);
    return () => clearTimeout(timer);
  }, [fen, enabled]);

  return enabled && threat?.fen === fen ? threat.uci : null;
};
