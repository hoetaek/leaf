import { useEffect, useRef, useState, useSyncExternalStore } from "react";
import { responseErrorMessage } from "./api";
import { getTick, markChanged, subscribe } from "./pollingClock";

// Re-render this consumer on every shared polling tick, so all mounted
// resources re-fetch together off one clock.
function usePollingTick(): number {
  return useSyncExternalStore(subscribe, getTick);
}

type ResourceState<T> = { data: T | null; error: string | null; path: string | null };

export function useJsonResource<T>(path: string | null) {
  const [resource, setResource] = useState<ResourceState<T>>({
    data: null,
    error: null,
    path: null,
  });
  const tick = usePollingTick();
  const lastTextRef = useRef<string | null>(null);
  const lastPathRef = useRef<string | null>(null);

  useEffect(() => {
    let alive = true;
    if (!path) return undefined;

    // A new path starts fresh: forget the previous path's body so its first
    // load always renders.
    if (lastPathRef.current !== path) {
      lastPathRef.current = path;
      lastTextRef.current = null;
    }

    fetch(path)
      .then(async (response) => {
        if (!response.ok) throw new Error(await responseErrorMessage(response));
        const text = await response.text();
        if (!alive) return;
        // Unchanged body → skip the state swap entirely. This keeps the data
        // reference identical, so memoized models and the d3 graph simulation
        // do not re-run on a no-op poll (scroll/zoom/panels stay put).
        if (text === lastTextRef.current) return;
        // A change to already-shown data (not the first load) is a poll-driven
        // update — signal it so the LiveIndicator can flash on real change only.
        const wasLoaded = lastTextRef.current !== null;
        lastTextRef.current = text;
        setResource({ data: JSON.parse(text) as T, error: null, path });
        if (wasLoaded) markChanged();
      })
      .catch((nextError) => {
        if (!alive) return;
        const message = nextError instanceof Error ? nextError.message : String(nextError);
        // A failed poll keeps the last good data; only an initial load (no data
        // yet) surfaces the error.
        setResource((prev) => (prev.data !== null && prev.path === path ? prev : { data: null, error: message, path }));
      });

    return () => {
      alive = false;
    };
  }, [path, tick]);

  return {
    data: resource.path === path ? resource.data : null,
    error: resource.path === path ? resource.error : null,
  };
}
