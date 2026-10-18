import { useEffect, useEffectEvent } from "react";
import { onAppEvent, type AppEventName, type AppEvents } from "../api/events";

/**
 * Listens for a backend event for as long as the component is mounted.
 *
 * Subscribed once, on mount — not again after every render. Resubscribing
 * drops whatever arrives in the gap between the old listener going and the
 * new one arriving, and for the live engine, where every snapshot is a
 * render, that gap was constant. `handler` is read through an effect event,
 * so it always sees the latest props and state without being a dependency;
 * an inline arrow function is fine.
 *
 * Each provider used to spell out the `listen` / `unlisten` pairing itself.
 */
export const useAppEvent = <E extends AppEventName>(
  event: E,
  handler: (payload: AppEvents[E]) => void,
): void => {
  const onEvent = useEffectEvent(handler);

  useEffect(() => {
    // `listen` resolves asynchronously, so the cleanup waits for it — an
    // unmount before it settles still unsubscribes once it does.
    const unlisten = onAppEvent(event, (payload) => onEvent(payload));
    return () => {
      unlisten.then((stop) => stop());
    };
  }, [event]);
};
