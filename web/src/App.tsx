import { useEffect, useState } from "react";
import { Toaster } from "./Toast";
import WorkspaceList from "./WorkspaceList";
import ReviewReader from "./ReviewReader";
import GraphView from "./GraphView";
import TreeView from "./TreeView";
import LiveIndicator from "./LiveIndicator";
import { useJsonResource } from "./useJsonResource";
import type { WorkspaceListResponse } from "./types";

// Tiny hash router: #/ (list) · #/tree · #/graph · #/leaf/<slug> · #/leaf/<slug>/ref/<path>
function isAppTextEntryElement(element: Element | null): boolean {
  const tagName = element?.tagName;
  return tagName === "INPUT" || tagName === "TEXTAREA" || tagName === "SELECT";
}

function useHashRoute(): string {
  const [hash, setHash] = useState(window.location.hash || "#/");
  useEffect(() => {
    const on = () => setHash(window.location.hash || "#/");
    window.addEventListener("hashchange", on);
    return () => window.removeEventListener("hashchange", on);
  }, []);
  return hash;
}

function useTopLevelShortcuts() {
  useEffect(() => {
    const onKey = (event: KeyboardEvent) => {
      if (isAppTextEntryElement(document.activeElement)) return;
      if (event.metaKey || event.ctrlKey || event.altKey) return;

      if (event.key === "1") {
        event.preventDefault();
        window.location.hash = "#/";
      } else if (event.key === "2") {
        event.preventDefault();
        window.location.hash = "#/graph";
      } else if (event.key === "3") {
        event.preventDefault();
        window.location.hash = "#/tree";
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, []);
}

export default function App() {
  const hash = useHashRoute();
  useTopLevelShortcuts();
  const { data: workspace } = useJsonResource<WorkspaceListResponse>("/api/list");
  const referenceMatch = hash.match(/^#\/leaf\/([^/]+)\/ref\/(.+)$/);
  const leafMatch = referenceMatch ? null : hash.match(/^#\/leaf\/(.+)$/);
  const view =
    referenceMatch || leafMatch
      ? "leaf"
      : hash.startsWith("#/graph")
        ? "graph"
        : hash.startsWith("#/tree")
          ? "tree"
          : "list";
  const leafSlug = referenceMatch
    ? decodeURIComponent(referenceMatch[1])
    : leafMatch
      ? decodeURIComponent(leafMatch[1])
      : null;
  const referencePath = referenceMatch ? decodeURIComponent(referenceMatch[2]) : undefined;

  return (
    <div className="shell">
      <header className="topbar">
        <a className="brand" href="#/">
          <span className="mk">&#9672;</span> LEAF
        </a>
        {workspace?.workspace_name && (
          <span className="workspace-name" title={workspace.workspace_name}>
            &middot; {workspace.workspace_name}
          </span>
        )}
        <nav className="tabs">
          <a aria-keyshortcuts="1" className={view === "list" ? "on" : ""} href="#/">
            Workspace
            <span aria-hidden="true" className="tabkey">
              1
            </span>
          </a>
          <a aria-keyshortcuts="2" className={view === "graph" ? "on" : ""} href="#/graph">
            Graph
            <span aria-hidden="true" className="tabkey">
              2
            </span>
          </a>
          <a aria-keyshortcuts="3" className={view === "tree" ? "on" : ""} href="#/tree">
            Tree
            <span aria-hidden="true" className="tabkey">
              3
            </span>
          </a>
        </nav>
        <LiveIndicator />
      </header>

      {view === "list" && <WorkspaceList />}
      {view === "leaf" && leafSlug && <ReviewReader slug={leafSlug} referencePath={referencePath} />}
      {view === "graph" && <GraphView />}
      {view === "tree" && <TreeView />}
      <Toaster />
    </div>
  );
}
