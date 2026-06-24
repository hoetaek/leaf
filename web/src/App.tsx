import { useEffect, useState } from "react";
import WorkspaceList from "./WorkspaceList";
import ReviewReader from "./ReviewReader";
import GraphView from "./GraphView";

// Tiny hash router: #/ (list) · #/leaf/<slug> (review reader) · #/graph
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
  const leafMatch = hash.match(/^#\/leaf\/(.+)$/);
  const view = leafMatch ? "leaf" : hash.startsWith("#/graph") ? "graph" : "list";
  const leafSlug = leafMatch ? decodeURIComponent(leafMatch[1]) : null;

  return (
    <div className="shell">
      <header className="topbar">
        <a className="brand" href="#/">
          <span className="mk">&#9672;</span> LEAF
        </a>
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
      {view === "leaf" && leafSlug && <ReviewReader slug={leafSlug} />}
      {view === "graph" && <GraphView />}
    </div>
  );
}
