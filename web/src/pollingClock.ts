// A single shared polling clock for the whole app. One timer drives every
// mounted `useJsonResource` to re-fetch together (see useJsonResource), and the
// LiveIndicator subscribes to the same clock. The clock is read-only: it only
// signals "time to re-ask", never touches `.leaf`.
//
// Lifecycle is ref-counted: the interval runs only while there is a subscriber
// AND auto-refresh is enabled AND the tab is visible, so unmount, the user
// toggle, and tab-hide each stop the timer rather than leaking it.

export const POLL_INTERVAL_MS = 5000;
const REFRESH_THROTTLE_MS = 300;
const ENABLED_KEY = "leaf-live-enabled";

type Listener = () => void;

function readEnabled(): boolean {
  try {
    return localStorage.getItem(ENABLED_KEY) !== "0";
  } catch {
    return true;
  }
}

function persistEnabled(value: boolean): void {
  try {
    localStorage.setItem(ENABLED_KEY, value ? "1" : "0");
  } catch {
    // storage unavailable (private mode etc.) — keep working in-memory
  }
}

const listeners = new Set<Listener>();
let tick = 0;
let lastTickAt = Date.now();
let changeSeq = 0;
let enabled = readEnabled();
let paused = false; // tab hidden
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

function shouldRun(): boolean {
  return enabled && !paused && listeners.size > 0;
}

function startTimer(): void {
  if (timerId !== null || !shouldRun()) return;
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

// Bumps when a mounted resource applies a real data change (poll-driven update
// to already-shown data). The LiveIndicator flashes on this, so motion marks an
// actual change rather than idle waiting.
export function getChangeSeq(): number {
  return changeSeq;
}

export function markChanged(): void {
  changeSeq += 1;
  emit();
}

export function isEnabled(): boolean {
  return enabled;
}

export function setEnabled(value: boolean): void {
  if (enabled === value) return;
  enabled = value;
  persistEnabled(value);
  if (value) {
    startTimer();
    refreshNow();
  } else {
    stopTimer();
  }
  emit();
}

export function toggleEnabled(): void {
  setEnabled(!enabled);
}

// Force an immediate tick (toggle-on / resume from hidden). Throttled so a rapid
// burst can't reset the interval into starvation.
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
  changeSeq = 0;
  enabled = true;
  paused = false;
  lastRefreshAt = 0;
  try {
    localStorage.removeItem(ENABLED_KEY);
  } catch {
    // ignore
  }
}
