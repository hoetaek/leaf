// A single shared polling clock for the whole app. One timer drives every
// mounted `useJsonResource` to re-fetch together (see useJsonResource), and the
// LiveIndicator subscribes to the same clock for its countdown. The clock is
// read-only: it only signals "time to re-ask", never touches `.leaf`.
//
// Lifecycle is ref-counted: the interval starts on the first subscriber and is
// cleared when the last one leaves, so unmount (and test cleanup) naturally stop
// the timer rather than leaking a module-global interval.

export const POLL_INTERVAL_MS = 5000;
const REFRESH_THROTTLE_MS = 300;

type Listener = () => void;

const listeners = new Set<Listener>();
let tick = 0;
let lastTickAt = Date.now();
let paused = false;
let timerId: ReturnType<typeof setInterval> | null = null;
let lastRefreshAt = 0;
let visibilityBound = false;

function emit(): void {
  for (const listener of listeners) listener();
}

function advance(): void {
  tick += 1;
  lastTickAt = Date.now();
  emit();
}

function startTimer(): void {
  if (timerId !== null || paused) return;
  timerId = setInterval(advance, POLL_INTERVAL_MS);
}

function stopTimer(): void {
  if (timerId !== null) {
    clearInterval(timerId);
    timerId = null;
  }
}

function handleVisibility(): void {
  if (typeof document === "undefined") return;
  if (document.hidden) {
    paused = true;
    stopTimer();
    emit();
  } else if (paused) {
    paused = false;
    startTimer();
    refreshNow();
  }
}

function bindVisibility(): void {
  if (visibilityBound || typeof document === "undefined") return;
  document.addEventListener("visibilitychange", handleVisibility);
  visibilityBound = true;
}

function unbindVisibility(): void {
  if (!visibilityBound || typeof document === "undefined") return;
  document.removeEventListener("visibilitychange", handleVisibility);
  visibilityBound = false;
}

export function subscribe(listener: Listener): () => void {
  listeners.add(listener);
  if (listeners.size === 1) {
    paused = typeof document !== "undefined" && document.hidden;
    bindVisibility();
    startTimer();
  }
  return () => {
    listeners.delete(listener);
    if (listeners.size === 0) {
      stopTimer();
      unbindVisibility();
    }
  };
}

export function getTick(): number {
  return tick;
}

export function getLastTickAt(): number {
  return lastTickAt;
}

export function isPaused(): boolean {
  return paused;
}

// Force an immediate tick (manual refresh / resume from hidden). Throttled so a
// rapid burst of clicks can't reset the interval into starvation.
export function refreshNow(): void {
  const now = Date.now();
  if (now - lastRefreshAt < REFRESH_THROTTLE_MS) return;
  lastRefreshAt = now;
  if (timerId !== null) {
    stopTimer();
    startTimer();
  }
  advance();
}

// Test-only: reset all module state and timers between tests.
export function __resetPollingClock(): void {
  stopTimer();
  unbindVisibility();
  listeners.clear();
  tick = 0;
  lastTickAt = Date.now();
  paused = false;
  lastRefreshAt = 0;
}
