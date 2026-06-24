import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, expect, test, vi } from "vitest";
import GraphView from "./GraphView";
import { mockJsonFetch } from "./test/mockFetch";

const graphData = {
  nodes: [
    {
      id: "leaf:a",
      slug: "alpha",
      title: "Alpha Leaf",
      description: "First graph leaf.",
      tags: ["graph"],
    },
    {
      id: "leaf:b",
      slug: "beta",
      title: "Beta Leaf",
      description: "Second graph leaf.",
      tags: ["ux"],
    },
    {
      id: "leaf:c",
      slug: "gamma",
      title: "Gamma Leaf",
      description: "Disconnected graph leaf.",
      tags: ["island"],
    },
  ],
  edges: [
    { source: "leaf:a", target: "leaf:b", predicate: "cites" },
    { source: "leaf:a", target: "missing", predicate: "hidden_edge" },
  ],
};

function installSvgShims() {
  Object.defineProperty(SVGSVGElement.prototype, "createSVGPoint", {
    configurable: true,
    value() {
      return {
        x: 0,
        y: 0,
        matrixTransform() {
          return { x: this.x, y: this.y };
        },
      } as SVGPoint;
    },
  });
  Object.defineProperty(SVGSVGElement.prototype, "getScreenCTM", {
    configurable: true,
    value() {
      return {
        inverse() {
          return {};
        },
      } as DOMMatrix;
    },
  });
  Object.defineProperty(Element.prototype, "setPointerCapture", {
    configurable: true,
    value: vi.fn(),
  });
  Object.defineProperty(Element.prototype, "releasePointerCapture", {
    configurable: true,
    value: vi.fn(),
  });
}

function installStorageShim() {
  const values = new Map<string, string>();
  Object.defineProperty(window, "localStorage", {
    configurable: true,
    value: {
      get length() {
        return values.size;
      },
      clear: vi.fn(() => values.clear()),
      getItem: vi.fn((key: string) => values.get(key) ?? null),
      key: vi.fn((index: number) => [...values.keys()][index] ?? null),
      removeItem: vi.fn((key: string) => values.delete(key)),
      setItem: vi.fn((key: string, value: string) => {
        values.set(key, value);
      }),
    } satisfies Storage,
  });
}

beforeEach(() => {
  installSvgShims();
  installStorageShim();
  vi.stubGlobal("fetch", mockJsonFetch({ "/api/graph": graphData }));
  vi.spyOn(window, "requestAnimationFrame").mockImplementation((callback) => {
    callback(0);
    return 1;
  });
  vi.spyOn(window, "cancelAnimationFrame").mockImplementation(() => undefined);
  window.location.hash = "#/graph";
});

test("renders graph counts, directed links, node details, and graph interactions", async () => {
  const { container } = render(<GraphView />);

  expect(screen.getByText("불러오는 중…")).toBeInTheDocument();
  expect(await screen.findByRole("heading", { name: "Knowledge graph" })).toBeInTheDocument();
  expect(screen.getByText(/nodes 3 · links 1 · hidden 1/)).toBeInTheDocument();
  expect(screen.getByRole("heading", { name: "Alpha Leaf" })).toBeInTheDocument();
  expect(screen.getByText("현재 graph에 없는 fallen 타깃 edge 1개는 숨겼습니다.")).toBeInTheDocument();

  await waitFor(() => expect(container.querySelectorAll(".graph-node")).toHaveLength(3));
  expect(screen.getByText("cites")).toBeInTheDocument();

  const nodes = container.querySelectorAll<SVGGElement>(".graph-node");
  fireEvent.mouseEnter(nodes[1]);
  expect(nodes[2]).toHaveClass("dim");
  fireEvent.mouseLeave(nodes[1]);

  fireEvent.pointerDown(nodes[1], { button: 0, pointerId: 7, clientX: 440, clientY: 260 });
  fireEvent.pointerUp(nodes[1], { pointerId: 7, clientX: 440, clientY: 260 });

  expect(await screen.findByRole("heading", { name: "Beta Leaf" })).toBeInTheDocument();
  expect(JSON.parse(window.localStorage.getItem("leaf-graph-visited") || "[]")).toEqual(["leaf:b"]);

  fireEvent.wheel(container.querySelector("svg")!, { deltaY: -120, clientX: 440, clientY: 260 });
  fireEvent.click(screen.getByRole("button", { name: "Reset" }));

  fireEvent.doubleClick(nodes[1]);
  expect(window.location.hash).toBe("#/leaf/beta");
});

test("renders empty and error graph states", async () => {
  vi.stubGlobal("fetch", mockJsonFetch({ "/api/graph": { nodes: [], edges: [] } }));
  const { unmount } = render(<GraphView />);

  expect(await screen.findByText("pressed leaf graph가 비어 있습니다")).toBeInTheDocument();

  unmount();
  vi.stubGlobal(
    "fetch",
    vi.fn(async () => new Response("Nope", { status: 500 })),
  );
  render(<GraphView />);

  expect(await screen.findByText(/그래프를 불러오지 못했습니다: HTTP 500/)).toBeInTheDocument();
});
