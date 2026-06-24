import { fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import { beforeEach, expect, test, vi } from "vitest";
import ReviewReader from "./ReviewReader";
import { mockJsonFetch } from "./test/mockFetch";

const reviewData = {
  slug: "web-graph-structure-refactor",
  sources: [
    {
      gate: "① Intent",
      phase: "Learn",
      relative_path: "01-Learn/01-intent.md",
      present: true,
      markdown: "# Intent\n\n그래프 구조를 분리한다.",
    },
    {
      gate: "② Unknowns",
      phase: "Learn",
      relative_path: "01-Learn/02-unknowns.md",
      present: false,
      markdown: "",
    },
  ],
  references: [
    { relative_path: "01-Learn/02-references/a.md", markdown: "# Alpha\n\n첫 번째 레퍼런스" },
    { relative_path: "01-Learn/02-references/b.md", markdown: "# Beta\n\n두 번째 레퍼런스" },
  ],
};

class MockIntersectionObserver implements IntersectionObserver {
  readonly root = null;
  readonly rootMargin = "";
  readonly scrollMargin = "";
  readonly thresholds = [];

  disconnect = vi.fn();
  observe = vi.fn();
  takeRecords = vi.fn(() => []);
  unobserve = vi.fn();
}

beforeEach(() => {
  vi.stubGlobal("fetch", mockJsonFetch({ "/api/review/web-graph-structure-refactor": reviewData }));
  vi.stubGlobal("IntersectionObserver", MockIntersectionObserver);
  vi.spyOn(window, "scrollBy").mockImplementation(() => undefined);
  Object.defineProperty(HTMLElement.prototype, "scrollBy", {
    configurable: true,
    value: vi.fn(),
  });
  Object.defineProperty(HTMLElement.prototype, "scrollIntoView", {
    configurable: true,
    value: vi.fn(),
  });
  window.location.hash = "#/leaf/web-graph-structure-refactor";
});

test("renders review gates, markdown, missing gate states, and reference drawer", async () => {
  render(<ReviewReader slug="web-graph-structure-refactor" />);

  expect(screen.getByText("불러오는 중…")).toBeInTheDocument();
  expect(await screen.findByText("web-graph-structure-refactor")).toBeInTheDocument();
  expect(screen.getAllByText("① Intent")[0]).toBeInTheDocument();
  expect(screen.getByText("그래프 구조를 분리한다.")).toBeInTheDocument();
  expect(screen.getByText("(이 게이트 문서는 아직 없음)")).toBeInTheDocument();

  fireEvent.click(screen.getByRole("button", { name: /Refs 2/ }));

  expect(screen.getByText("References")).toBeInTheDocument();
  expect(screen.getByText("a.md")).toBeInTheDocument();
  expect(screen.getByText("첫 번째 레퍼런스")).toBeInTheDocument();

  fireEvent.click(screen.getByText("b.md"));
  expect(screen.getByText("두 번째 레퍼런스")).toBeInTheDocument();

  fireEvent.click(screen.getByRole("button", { name: "전체 페이지로 보기" }));
  expect(window.location.hash).toBe("#/leaf/web-graph-structure-refactor/ref/01-Learn%2F02-references%2Fb.md");
});

test("opens the mobile table of contents and follows reader keyboard shortcuts", async () => {
  const { container } = render(<ReviewReader slug="web-graph-structure-refactor" />);
  await screen.findByText("그래프 구조를 분리한다.");

  fireEvent.click(screen.getByRole("button", { name: /① Intent.*목차/ }));
  expect(screen.getByText("Gates")).toBeInTheDocument();
  fireEvent.click(within(container.querySelector(".toc-overlay")!).getByText("② Unknowns"));
  await waitFor(() => expect(screen.queryByText("Gates")).not.toBeInTheDocument());

  fireEvent.keyDown(window, { key: "R" });
  expect(screen.getByText("References")).toBeInTheDocument();
  expect(screen.getByText(/이동/)).toBeInTheDocument();

  fireEvent.keyDown(window, { key: "l" });
  expect(screen.getByText(/스크롤/)).toBeInTheDocument();

  fireEvent.keyDown(window, { key: "f" });
  expect(window.location.hash).toBe("#/leaf/web-graph-structure-refactor/ref/01-Learn%2F02-references%2Fa.md");
  window.location.hash = "#/leaf/web-graph-structure-refactor";

  fireEvent.keyDown(window, { key: "h" });
  expect(screen.getByText(/이동/)).toBeInTheDocument();

  fireEvent.keyDown(window, { key: "h" });
  await waitFor(() => expect(screen.queryByText("References")).not.toBeInTheDocument());

  fireEvent.keyDown(window, { key: "q" });
  expect(window.location.hash).toBe("#/");
});

test("renders a selected reference as a full page", async () => {
  render(<ReviewReader slug="web-graph-structure-refactor" referencePath="01-Learn/02-references/b.md" />);

  expect(await screen.findByRole("heading", { name: "Beta" })).toBeInTheDocument();
  expect(screen.getByText("두 번째 레퍼런스")).toBeInTheDocument();
  expect(screen.getByText("01-Learn/02-references/b.md")).toBeInTheDocument();
  expect(screen.getByRole("link", { name: /review로 돌아가기/ })).toHaveAttribute(
    "href",
    "#/leaf/web-graph-structure-refactor",
  );
});

test("keeps j and k as vertical scroll keys on full page references", async () => {
  window.location.hash = "#/leaf/web-graph-structure-refactor/ref/01-Learn%2F02-references%2Fa.md";
  render(<ReviewReader slug="web-graph-structure-refactor" referencePath="01-Learn/02-references/a.md" />);

  expect(await screen.findByRole("heading", { name: "Alpha" })).toBeInTheDocument();

  fireEvent.keyDown(window, { key: "j" });
  expect(window.scrollBy).toHaveBeenCalledWith({ top: 90, behavior: "smooth" });
  expect(window.location.hash).toBe("#/leaf/web-graph-structure-refactor/ref/01-Learn%2F02-references%2Fa.md");

  fireEvent.keyDown(window, { key: "k" });
  expect(window.scrollBy).toHaveBeenLastCalledWith({ top: -90, behavior: "smooth" });
  expect(window.location.hash).toBe("#/leaf/web-graph-structure-refactor/ref/01-Learn%2F02-references%2Fa.md");
});

test("moves between full page references with h and l", async () => {
  const { rerender } = render(
    <ReviewReader slug="web-graph-structure-refactor" referencePath="01-Learn/02-references/a.md" />,
  );

  expect(await screen.findByRole("heading", { name: "Alpha" })).toBeInTheDocument();

  fireEvent.keyDown(window, { key: "l" });
  expect(window.location.hash).toBe("#/leaf/web-graph-structure-refactor/ref/01-Learn%2F02-references%2Fb.md");

  rerender(<ReviewReader slug="web-graph-structure-refactor" referencePath="01-Learn/02-references/b.md" />);
  expect(await screen.findByRole("heading", { name: "Beta" })).toBeInTheDocument();

  fireEvent.keyDown(window, { key: "h" });
  expect(window.location.hash).toBe("#/leaf/web-graph-structure-refactor/ref/01-Learn%2F02-references%2Fa.md");
});

test("renders reader validation and API error states", async () => {
  const { rerender } = render(<ReviewReader slug="" />);
  expect(screen.getByText("리뷰 slug가 없습니다.")).toBeInTheDocument();

  vi.stubGlobal(
    "fetch",
    vi.fn(async () => new Response("Nope", { status: 500 })),
  );
  rerender(<ReviewReader slug="broken" />);

  expect(await screen.findByText(/리뷰를 불러오지 못했습니다: HTTP 500/)).toBeInTheDocument();
});
