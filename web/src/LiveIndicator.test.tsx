import { act, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, beforeEach, expect, test, vi } from "vitest";
import LiveIndicator from "./LiveIndicator";
import { __resetPollingClock, isEnabled, markChanged } from "./pollingClock";

beforeEach(() => {
  vi.useFakeTimers();
  __resetPollingClock();
});

afterEach(() => {
  __resetPollingClock();
  vi.useRealTimers();
});

test("is calm by default: steady dot + last refresh time, no seconds number, no bar", () => {
  const { container } = render(<LiveIndicator />);

  expect(screen.getByText(/갱신 \d{2}:\d{2}:\d{2}/)).toBeInTheDocument();
  expect(screen.queryByText(/\d+s\b/)).not.toBeInTheDocument();
  expect(container.querySelector(".live-bar")).toBeNull();

  const dot = container.querySelector(".live-dot");
  expect(dot).not.toBeNull();
  expect(dot?.classList.contains("off")).toBe(false);
  expect(dot?.classList.contains("flash")).toBe(false); // calm until a real change
});

test("clicking toggles auto-refresh off, then back on", () => {
  render(<LiveIndicator />);
  const button = screen.getByRole("button");
  expect(isEnabled()).toBe(true);
  expect(button).toHaveAttribute("aria-pressed", "true");

  act(() => {
    fireEvent.click(button);
  });
  expect(isEnabled()).toBe(false);
  expect(screen.getByText("자동 갱신 꺼짐")).toBeInTheDocument();
  expect(button).toHaveAttribute("aria-pressed", "false");

  act(() => {
    fireEvent.click(button);
  });
  expect(isEnabled()).toBe(true);
  expect(screen.getByText(/갱신 \d{2}:\d{2}:\d{2}/)).toBeInTheDocument();
});

test("flashes the dot only after a real data change", () => {
  const { container } = render(<LiveIndicator />);
  expect(container.querySelector(".live-dot.flash")).toBeNull();

  act(() => {
    markChanged();
  });

  expect(container.querySelector(".live-dot.flash")).not.toBeNull();
});
