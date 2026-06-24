import { useEffect, useState } from "react";
import { cachedWorkspacePreview, loadWorkspacePreview } from "./workspacePreviewCache";
import type { WorkspacePreviewLine, WorkspacePreviewResponse, WorkspaceRow } from "./types";

type PreviewState =
  | { status: "idle"; data: null; error: null }
  | { status: "loading"; data: WorkspacePreviewResponse | null; error: null }
  | { status: "ready"; data: WorkspacePreviewResponse; error: null }
  | { status: "error"; data: null; error: string };

type PreviewResult =
  | { slug: string; status: "ready"; data: WorkspacePreviewResponse; error: null }
  | { slug: string; status: "error"; data: null; error: string };

function PreviewLine({ line, index }: { line: WorkspacePreviewLine; index: number }) {
  if (line.kind === "source_boundary") {
    return (
      <div className="wprev-source">
        <span className="wprev-phase">{line.phase}</span>
        <b>{line.gate}</b>
        <em>{line.source}</em>
      </div>
    );
  }

  if (line.kind === "heading") {
    return <h4 className={`wprev-heading level-${Math.min(Math.max(line.level, 1), 6)}`}>{line.text}</h4>;
  }

  if (line.kind === "checkbox") {
    return (
      <div className="wprev-list check">
        <span className="wprev-marker">{line.marker}</span>
        <span className="wprev-check">{line.checked ? "[x]" : "[ ]"}</span>
        <span>{line.text}</span>
      </div>
    );
  }

  if (line.kind === "list_item") {
    return (
      <div className="wprev-list item">
        <span className="wprev-marker">{line.marker}</span>
        <span>{line.text}</span>
      </div>
    );
  }

  if (line.kind === "code" || line.kind === "table") {
    return <pre className="wprev-code">{line.text || " "}</pre>;
  }

  if (!line.text.trim()) {
    return <div aria-hidden="true" className="wprev-gap" />;
  }

  return <p className={index === 0 ? "wprev-text first" : "wprev-text"}>{line.text}</p>;
}

export default function WorkspacePreview({ row }: { row?: WorkspaceRow }) {
  const [result, setResult] = useState<PreviewResult | null>(null);
  const slug = row?.slug;

  useEffect(() => {
    if (!slug || cachedWorkspacePreview(slug)) return undefined;

    let alive = true;
    loadWorkspacePreview(slug)
      .then((data) => {
        if (!alive) return;
        setResult({ slug, status: "ready", data, error: null });
      })
      .catch((error) => {
        if (!alive) return;
        setResult({ slug, status: "error", data: null, error: error instanceof Error ? error.message : String(error) });
      });

    return () => {
      alive = false;
    };
  }, [slug]);

  const cached = slug ? cachedWorkspacePreview(slug) : undefined;
  const state: PreviewState = !slug
    ? { status: "idle", data: null, error: null }
    : cached
      ? { status: "ready", data: cached, error: null }
      : result?.slug === slug
        ? result
        : { status: "loading", data: null, error: null };

  const title = state.data?.title || row?.slug || "Preview";
  const path = typeof row?.path === "string" ? row.path : undefined;

  return (
    <aside className="wprev" aria-label="Workspace preview">
      <div className="wprev-head">
        <div>
          <h3>{title}</h3>
          {path && <p>{path}</p>}
        </div>
      </div>

      {state.status === "idle" && <p className="muted">선택된 leaf가 없습니다.</p>}
      {state.status === "loading" && <p className="muted">preview 불러오는 중...</p>}
      {state.status === "error" && <p className="err">preview를 불러오지 못했습니다: {state.error}</p>}
      {state.status === "ready" && (
        <div className="wprev-body">
          {state.data.lines.map((line, index) => (
            <PreviewLine key={`${line.kind}-${index}`} line={line} index={index} />
          ))}
        </div>
      )}
    </aside>
  );
}
