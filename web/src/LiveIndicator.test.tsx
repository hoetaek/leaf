import { act, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, beforeEach, expect, test, vi } from "vitest";
import LiveIndicator from "./LiveIndicator";
import { __resetPollingClock, getTick } from "./pollingClock";

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

test("counts down toward the next auto-refresh", () => {
  render(<LiveIndicator />);

  expect(screen.getByLabelText("지금 새로고침")).toBeInTheDocument();
  expect(screen.getByText(/5s/)).toBeInTheDocument();

  act(() => {
    vi.advanceTimersByTime(1000);
  });
  expect(screen.getByText(/4s/)).toBeInTheDocument();
});

test("refreshes immediately when clicked", () => {
  render(<LiveIndicator />);
  const start = getTick();

  act(() => {
    fireEvent.click(screen.getByLabelText("지금 새로고침"));
  });

  expect(getTick()).toBe(start + 1);
});

test("shows a paused state while the tab is hidden", () => {
  render(<LiveIndicator />);

  act(() => {
    setHidden(true);
  });

  expect(screen.getByText("paused")).toBeInTheDocument();
});
