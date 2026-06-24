import { render, screen } from "@testing-library/react";
import { expect, test, vi } from "vitest";
import { useJsonResource } from "./useJsonResource";

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
