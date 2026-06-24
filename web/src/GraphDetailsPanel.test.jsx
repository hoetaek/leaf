import { render, screen } from "@testing-library/react";
import { expect, test } from "vitest";
import GraphDetailsPanel from "./GraphDetailsPanel.jsx";

const selected = {
  id: "leaf:react-lint-format-baseline",
  slug: "react-lint-format-baseline",
  title: "React lint format baseline",
  degree: 3,
  tags: ["web", "quality"],
  description: "React quality gate for the web UI.",
};

test("renders selected leaf details with a leaf link", () => {
  render(<GraphDetailsPanel selected={selected} hiddenEdgeCount={2} />);

  expect(screen.getByRole("heading", { name: selected.title })).toBeInTheDocument();
  expect(screen.getByText("degree 3")).toBeInTheDocument();
  expect(screen.getByText("tags 2")).toBeInTheDocument();
  expect(screen.getByText("#web")).toBeInTheDocument();
  expect(screen.getByText("#quality")).toBeInTheDocument();
  expect(screen.getByRole("link", { name: /본문 열기/ })).toHaveAttribute("href", "#/leaf/react-lint-format-baseline");
  expect(screen.getByText("현재 graph에 없는 fallen 타깃 edge 2개는 숨겼습니다.")).toBeInTheDocument();
});

test("renders an empty selection prompt", () => {
  render(<GraphDetailsPanel selected={null} hiddenEdgeCount={0} />);

  expect(screen.getByText("노드를 선택하세요.")).toBeInTheDocument();
  expect(screen.queryByRole("link")).not.toBeInTheDocument();
});
