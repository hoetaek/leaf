import { afterEach, beforeEach, expect, test, vi } from "vitest";
import {
  __resetPollingClock,
  getChangeSeq,
  getTick,
  isEnabled,
  markChanged,
  POLL_INTERVAL_MS,
  refreshNow,
  setEnabled,
  subscribe,
} from "./pollingClock";

function setHidden(hidden: boolean) {
  Object.defineProperty(document, "hidden", { configurable: true, value: hidden });
  document.dispatchEvent(new Event("visibilitychange"));
}

beforeEach(() => {
  vi.useFakeTimers();
  __resetPollingClock();
});

afterEach(() => {
  __resetPollingClock();
  setHidden(false);
  vi.useRealTimers();
});

test("advances the tick once per interval while subscribed and enabled", () => {
  const unsubscribe = subscribe(() => {});
  const start = getTick();

  vi.advanceTimersByTime(POLL_INTERVAL_MS);
  expect(getTick()).toBe(start + 1);

  vi.advanceTimersByTime(POLL_INTERVAL_MS);
  expect(getTick()).toBe(start + 2);

  unsubscribe();
});

test("stops ticking once the last subscriber leaves (ref-counted)", () => {
  const unsubscribe = subscribe(() => {});
  vi.advanceTimersByTime(POLL_INTERVAL_MS);
  const afterOneTick = getTick();

  unsubscribe();
  vi.advanceTimersByTime(POLL_INTERVAL_MS * 3);

  expect(getTick()).toBe(afterOneTick);
});

test("disabling stops polling; re-enabling resumes and persists the choice", () => {
  subscribe(() => {});
  const start = getTick();

  setEnabled(false);
  expect(isEnabled()).toBe(false);
  expect(localStorage.getItem("leaf-live-enabled")).toBe("0");
  vi.advanceTimersByTime(POLL_INTERVAL_MS * 3);
  expect(getTick()).toBe(start); // no ticks while disabled

  setEnabled(true);
  expect(isEnabled()).toBe(true);
  expect(localStorage.getItem("leaf-live-enabled")).toBe("1");
  expect(getTick()).toBe(start + 1); // immediate refresh on enable
});

test("refreshNow throttles a rapid burst into one tick", () => {
  subscribe(() => {});
  const start = getTick();

  refreshNow();
  refreshNow();
  refreshNow();

  expect(getTick()).toBe(start + 1);
});

test("markChanged increments the change sequence", () => {
  const start = getChangeSeq();

  markChanged();
  markChanged();

  expect(getChangeSeq()).toBe(start + 2);
});

test("pauses while the tab is hidden and refreshes on resume", () => {
  subscribe(() => {});
  const start = getTick();

  setHidden(true);
  vi.advanceTimersByTime(POLL_INTERVAL_MS * 3);
  expect(getTick()).toBe(start); // no ticks while hidden

  setHidden(false);
  expect(getTick()).toBe(start + 1); // immediate refresh on resume
});
