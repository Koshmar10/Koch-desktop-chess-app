import { useState } from "react";
import type { EngineSnapshot } from "../../api/bindings/EngineSnapshot";
import { useAppEvent } from "../../app/useAppEvent";

/**
 * The live engine's latest full set of lines, for whichever position it was
 * last searching — the caller checks its `position_key`.
 */
export const useLiveEngine = (): EngineSnapshot | null => {
  const [liveSnapshot, setLiveSnapshot] = useState<EngineSnapshot | null>(null);
  useAppEvent("pvline", setLiveSnapshot);
  return liveSnapshot;
};
