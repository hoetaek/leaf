import { render, screen, waitFor } from "@testing-library/react";
import { beforeEach, expect, test, vi } from "vitest";
import App from "./App";

vi.mock("./WorkspaceList", () => ({
  default: () => <div>workspace route</div>,
}));

vi.mock("./GraphView", () => ({
  default: () => <div>graph route</div>,
}));

vi.mock("./ReviewReader", () => ({
  default: ({ slug }: { slug: string }) => <div>review route: {slug}</div>,
}));

beforeEach(() => {
  window.location.hash = "#/";
});

test("routes between workspace, graph, and leaf review hash views", async () => {
  render(<App />);

  expect(screen.getByText("workspace route")).toBeInTheDocument();
  expect(screen.getByRole("link", { name: /LEAF/ })).toHaveAttribute("href", "#/");
  expect(screen.getByRole("link", { name: "Workspace" })).toHaveClass("on");

  window.location.hash = "#/graph";
  window.dispatchEvent(new HashChangeEvent("hashchange"));
  await waitFor(() => expect(screen.getByText("graph route")).toBeInTheDocument());
  expect(screen.getByRole("link", { name: "Graph" })).toHaveClass("on");

  window.location.hash = "#/leaf/web-graph-structure-refactor";
  window.dispatchEvent(new HashChangeEvent("hashchange"));
  await waitFor(() => expect(screen.getByText("review route: web-graph-structure-refactor")).toBeInTheDocument());
});
