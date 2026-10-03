import { useEffect, useEffectEvent, useRef, useState } from "react";
import type { LiveEngineSettings } from "../../../../api/bindings/LiveEngineSettings";
import {
  applyLiveEngineSettings,
  getLiveEngineSettings,
} from "../../../../api/analyzer";

// Applying stops the search and restarts it, so wait for a pause instead
// of restarting on every click of a spinner.
const APPLY_DEBOUNCE_MS = 600;

// Stockfish's own defaults — what a fresh session runs with — shown until
// the session's real values load.
const STOCKFISH_DEFAULTS: LiveEngineSettings = {
  multi_pv: 1,
  threads: 1,
  hash_mb: 16,
};

/**
 * The live engine's options, read from the running session and applied to
 * it as they change. Never saved: they last as long as the app runs, and
 * stay separate from the saved analyzer settings the post-game pass uses.
 *
 * Applying stops the search, so `onApplied` runs once the new options are
 * in, to pick it back up.
 */
export const useLiveEngineSettings = (onApplied: () => void) => {
  const [settings, setSettings] =
    useState<LiveEngineSettings>(STOCKFISH_DEFAULTS);
  // The exact object last known to match the engine. Compared by
  // reference, which is enough because every edit builds a new object — it
  // stops the freshly loaded values being applied straight back.
  const applied = useRef<LiveEngineSettings | null>(null);
  // An effect event, so the latest `onApplied` is called without being a
  // dependency: it's a new function every render, and as one it would
  // restart the debounce each time a snapshot re-renders the screen.
  const notifyApplied = useEffectEvent(onApplied);

  useEffect(() => {
    getLiveEngineSettings()
      .then((current) => {
        applied.current = current;
        setSettings(current);
      })
      .catch(console.error);
  }, []);

  useEffect(() => {
    if (applied.current === null || settings === applied.current) return;

    const timer = setTimeout(() => {
      applied.current = settings;
      applyLiveEngineSettings(settings)
        .then(() => notifyApplied())
        .catch(console.error);
    }, APPLY_DEBOUNCE_MS);
    return () => clearTimeout(timer);
  }, [settings]);

  return [settings, setSettings] as const;
};
