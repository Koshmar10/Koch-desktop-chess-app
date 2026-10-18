import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { AnalysisStatus } from "./bindings/AnalysisStatus";
import type { EngineSnapshot } from "./bindings/EngineSnapshot";
import type { GameAnalysis } from "./bindings/GameAnalysis";
import type { GameStateView } from "./bindings/GameStateView";

/**
 * Every event the backend pushes, each with the payload it carries — the
 * same list as `src-tauri/src/app/events.rs`, and changed together with
 * it: a name here that the backend doesn't emit is a listener that never
 * fires, with nothing to say so.
 *
 * Pairing the name with its payload in one place means a listener can't
 * read one event's payload as another's, and can't misspell a name — both
 * fail to compile.
 */
export interface AppEvents {
  "engine-move-complete": GameStateView;
  "analysis-status": AnalysisStatus;
  // How far the running analysis pass is, 0–100.
  "update-analysis-progress": number;
  "game-analysis-complete": GameAnalysis;
  pvline: EngineSnapshot;
}

export type AppEventName = keyof AppEvents;

/**
 * Calls `handler` with each payload of `event` until the returned function
 * is called. Inside a component, use `useAppEvent`, which does that
 * unsubscribing for you.
 */
export const onAppEvent = <E extends AppEventName>(
  event: E,
  handler: (payload: AppEvents[E]) => void,
): Promise<UnlistenFn> =>
  listen<AppEvents[E]>(event, (message) => handler(message.payload));
