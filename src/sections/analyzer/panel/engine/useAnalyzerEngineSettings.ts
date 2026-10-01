import { useEffect, useRef, useState } from "react";
import type { AnalyzerEngineSettings } from "../../../../api/bindings/AnalyzerEngineSettings";
import {
  DEFAULT_ANALYZER_SETTINGS,
  getAnalyzerEngineSettings,
  saveAnalyzerEngineSettings,
} from "../../../../api/settings";

// Settings rows are append-only, so one write per keystroke would leave a
// row behind every click of a spinner. Waiting for a pause collapses an
// adjustment into a single snapshot.
const PERSIST_DEBOUNCE_MS = 600;

/**
 * The analyzer's engine settings, loaded from the same row the Settings
 * screen edits and written back as they change.
 *
 * Applying them to a running search is KOCH-10's job — there's no session
 * to re-issue `go` to yet. Until then this persists the intent, so the
 * values are already correct when the session arrives.
 */
export const useAnalyzerEngineSettings = () => {
  const [settings, setSettings] = useState<AnalyzerEngineSettings>(
    DEFAULT_ANALYZER_SETTINGS,
  );
  // The exact object last known to match what's on disk. Compared by
  // reference, which is enough because every edit below builds a new
  // object — it stops the freshly-loaded values being written straight
  // back as if the user had just typed them.
  const persisted = useRef<AnalyzerEngineSettings | null>(null);

  useEffect(() => {
    getAnalyzerEngineSettings()
      .then((saved) => {
        const loaded = saved ?? DEFAULT_ANALYZER_SETTINGS;
        persisted.current = loaded;
        setSettings(loaded);
      })
      .catch((err) => {
        console.error("get_analyzer_engine_settings failed", err);
        // Editing still persists after a failed read; the alternative is
        // a panel whose controls silently do nothing.
        persisted.current = DEFAULT_ANALYZER_SETTINGS;
      });
  }, []);

  useEffect(() => {
    if (persisted.current === null) return;
    if (settings === persisted.current) return;

    const timer = setTimeout(() => {
      persisted.current = settings;
      saveAnalyzerEngineSettings(settings).catch((err) =>
        console.error("save_analyzer_engine_settings failed", err),
      );
    }, PERSIST_DEBOUNCE_MS);
    return () => clearTimeout(timer);
  }, [settings]);

  return [settings, setSettings] as const;
};
