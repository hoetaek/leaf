import { vi } from "vitest";

export function mockJsonFetch(routes: Record<string, unknown>) {
  return vi.fn(async (input: RequestInfo | URL) => {
    const path = typeof input === "string" ? input : input instanceof URL ? input.pathname : input.url;
    const data = routes[path];

    if (data === undefined) {
      return new Response("Not found", { status: 404 });
    }

    return new Response(JSON.stringify(data), {
      status: 200,
      headers: { "Content-Type": "application/json" },
    });
  });
}

// Like mockJsonFetch, but each path returns the next body on successive calls,
// staying on the last entry once exhausted. Used to drive polling tests where a
// resource changes (or stays the same) between ticks.
export function mockJsonFetchSequence(routes: Record<string, unknown[]>) {
  const counts: Record<string, number> = {};
  return vi.fn(async (input: RequestInfo | URL) => {
    const path = typeof input === "string" ? input : input instanceof URL ? input.pathname : input.url;
    const sequence = routes[path];

    if (sequence === undefined || sequence.length === 0) {
      return new Response("Not found", { status: 404 });
    }

    const index = Math.min(counts[path] ?? 0, sequence.length - 1);
    counts[path] = (counts[path] ?? 0) + 1;

    return new Response(JSON.stringify(sequence[index]), {
      status: 200,
      headers: { "Content-Type": "application/json" },
    });
  });
}
