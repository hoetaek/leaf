import { useSyncExternalStore } from "react";
import { getChangeSeq, getLastTickAt, getTick, isEnabled, subscribe, toggleEnabled } from "./pollingClock";

function formatClock(ms: number): string {
  const date = new Date(ms);
  const pad = (n: number) => String(n).padStart(2, "0");
  return `${pad(date.getHours())}:${pad(date.getMinutes())}:${pad(date.getSeconds())}`;
}

// Topbar "live" affordance for the shared polling clock. Calm by default: a
// steady status dot plus the last refresh time, no idle motion. The dot flashes
// only when a poll actually changes the data. Clicking toggles auto-refresh
// on/off (persisted). Read-only — it only nudges the clock.
export default function LiveIndicator() {
  const enabled = useSyncExternalStore(subscribe, isEnabled);
  const changeSeq = useSyncExternalStore(subscribe, getChangeSeq);
  useSyncExternalStore(subscribe, getTick); // re-render each poll to keep the shown time current

  const dotClass = !enabled ? "live-dot off" : changeSeq > 0 ? "live-dot flash" : "live-dot";

  return (
    <button
      type="button"
      className={enabled ? "live-indicator" : "live-indicator off"}
      onClick={() => toggleEnabled()}
      aria-pressed={enabled}
      aria-label={enabled ? "자동 갱신 켜짐 — 끄려면 클릭" : "자동 갱신 꺼짐 — 켜려면 클릭"}
      title={enabled ? "자동 갱신 켜짐 · 클릭하면 끄기" : "자동 갱신 꺼짐 · 클릭하면 켜기"}
    >
      {/* key={changeSeq} remounts the dot only on a real data change, so the
          flash marks an actual update, not idle waiting. */}
      <span key={changeSeq} className={dotClass} aria-hidden="true" />
      <span className="live-text">{enabled ? `갱신 ${formatClock(getLastTickAt())}` : "자동 갱신 꺼짐"}</span>
    </button>
  );
}
