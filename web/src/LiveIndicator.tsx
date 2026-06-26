import { useEffect, useState, useSyncExternalStore } from "react";
import { getLastTickAt, getTick, isPaused, POLL_INTERVAL_MS, refreshNow, subscribe } from "./pollingClock";

function formatClock(ms: number): string {
  const date = new Date(ms);
  const pad = (n: number) => String(n).padStart(2, "0");
  return `${pad(date.getHours())}:${pad(date.getMinutes())}:${pad(date.getSeconds())}`;
}

const MAX_SECONDS = Math.round(POLL_INTERVAL_MS / 1000);

// Topbar "live" affordance for the shared polling clock: shows the countdown to
// the next auto-refresh and the last refresh time, spins on each tick, and lets
// the user refresh now. Read-only — it only nudges the clock.
export default function LiveIndicator() {
  const tick = useSyncExternalStore(subscribe, getTick);
  const paused = useSyncExternalStore(subscribe, isPaused);
  const [now, setNow] = useState(() => Date.now());

  // 1s display timer drives the countdown re-render — only while live. (setState
  // lives in the timer callback, never synchronously in the effect body.)
  useEffect(() => {
    if (paused) return undefined;
    const id = setInterval(() => setNow(Date.now()), 1000);
    return () => clearInterval(id);
  }, [paused]);

  const secondsToNext = Math.min(
    MAX_SECONDS,
    Math.max(0, Math.ceil((POLL_INTERVAL_MS - (now - getLastTickAt())) / 1000)),
  );

  return (
    <button
      type="button"
      className="live-indicator"
      onClick={() => refreshNow()}
      aria-label="지금 새로고침"
      title={
        paused
          ? "자동 갱신 일시정지 (탭 비활성) · 클릭하면 즉시 갱신"
          : `다음 갱신까지 ${secondsToNext}초 · 클릭하면 즉시 갱신`
      }
    >
      {/* key={tick} remounts the glyph each tick, replaying the CSS spin once. */}
      <span key={tick} className="live-glyph spin" aria-hidden="true">
        &#10227;
      </span>
      {paused ? (
        <span className="live-text">paused</span>
      ) : (
        <span className="live-text">
          {secondsToNext}s &middot; {formatClock(getLastTickAt())}
        </span>
      )}
    </button>
  );
}
