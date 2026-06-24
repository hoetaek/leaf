import { useEffect, useState } from "react";
import WorkspaceList from "./WorkspaceList";
import ReviewReader from "./ReviewReader";
import GraphView from "./GraphView";
import { useJsonResource } from "./useJsonResource";
import type { WorkspaceListResponse } from "./types";

// Tiny hash router: #/ (list) · #/leaf/<slug> (review reader) · #/leaf/<slug>/ref/<path> · #/graph
function useHashRoute(): string {
  const [hash, setHash] = useState(window.location.hash || "#/");
  useEffect(() => {
    const on = () => setHash(window.location.hash || "#/");
    window.addEventListener("hashchange", on);
    return () => window.removeEventListener("hashchange", on);
  }, []);
  return hash;
}

export default function App() {
  const hash = useHashRoute();
  const { data: workspace } = useJsonResource<WorkspaceListResponse>("/api/list");
  const referenceMatch = hash.match(/^#\/leaf\/([^/]+)\/ref\/(.+)$/);
  const leafMatch = referenceMatch ? null : hash.match(/^#\/leaf\/(.+)$/);
  const view = referenceMatch || leafMatch ? "leaf" : hash.startsWith("#/graph") ? "graph" : "list";
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
          <a className={view === "list" ? "on" : ""} href="#/">
            Workspace
          </a>
          <a className={view === "graph" ? "on" : ""} href="#/graph">
            Graph
          </a>
        </nav>
      </header>

      {view === "list" && <WorkspaceList />}
      {view === "leaf" && leafSlug && <ReviewReader slug={leafSlug} referencePath={referencePath} />}
      {view === "graph" && <GraphView />}
    </div>
  );
}
