import { act, render, screen } from "@testing-library/react";
import { useEffect } from "react";
import { afterEach, expect, test, vi } from "vitest";
import { useJsonResource } from "./useJsonResource";
import { __resetPollingClock, POLL_INTERVAL_MS } from "./pollingClock";
import { mockJsonFetchSequence } from "./test/mockFetch";

function deferred<T>() {
  let resolve!: (value: T) => void;
  const promise = new Promise<T>((nextResolve) => {
    resolve = nextResolve;
  });
  return { promise, resolve };
}

function jsonResponse(body: unknown) {
  return new Response(JSON.stringify(body), {
    headers: { "Content-Type": "application/json" },
  });
}

function Probe({ path }: { path: string }) {
  const { data, error } = useJsonResource<{ label: string }>(path);
  if (error) return <p>{error}</p>;
  return <p>{data?.label || "loading"}</p>;
}

test("shows loading instead of stale data when the resource path changes", async () => {
  const first = deferred<Response>();
  const second = deferred<Response>();
  vi.stubGlobal(
    "fetch",
    vi.fn((path: string) => (path === "/first" ? first.promise : second.promise)),
  );

  const { rerender } = render(<Probe path="/first" />);
  expect(screen.getByText("loading")).toBeInTheDocument();

  first.resolve(jsonResponse({ label: "first" }));
  expect(await screen.findByText("first")).toBeInTheDocument();

  rerender(<Probe path="/second" />);
  expect(screen.getByText("loading")).toBeInTheDocument();

  second.resolve(jsonResponse({ label: "second" }));
  expect(await screen.findByText("second")).toBeInTheDocument();
});

afterEach(() => {
  __resetPollingClock();
  vi.useRealTimers();
});

// C3: an unchanged poll must not produce a new data reference, so memoized
// consumers (the d3 graph model) never re-run on a no-op tick. A changed body
// must swap the reference.
test("keeps the data reference stable across an unchanged poll, swaps it on change", async () => {
  vi.useFakeTimers();
  const refs: Array<{ label: string }> = [];
  vi.stubGlobal("fetch", mockJsonFetchSequence({ "/r": [{ label: "a" }, { label: "a" }, { label: "b" }] }));

  function RefProbe() {
    const { data } = useJsonResource<{ label: string }>("/r");
    useEffect(() => {
      if (data) refs.push(data);
    }, [data]);
    return <p>{data?.label ?? "loading"}</p>;
  }

  render(<RefProbe />);

  // flush the initial mount fetch (no timer involved)
  await act(async () => {
    await vi.advanceTimersByTimeAsync(0);
  });
  expect(screen.getByText("a")).toBeInTheDocument();
  expect(refs).toHaveLength(1);

  // tick 1: identical body → skip the state swap, reference unchanged
  await act(async () => {
    await vi.advanceTimersByTimeAsync(POLL_INTERVAL_MS);
  });
  expect(refs).toHaveLength(1);

  // tick 2: changed body → new reference and visible update
  await act(async () => {
    await vi.advanceTimersByTimeAsync(POLL_INTERVAL_MS);
  });
  expect(screen.getByText("b")).toBeInTheDocument();
  expect(refs).toHaveLength(2);
});
