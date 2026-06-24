import { useCallback, useEffect, useMemo, useRef, useState, type MouseEvent } from "react";
import { leafHref, openLeaf } from "./routes";
import { useJsonResource } from "./useJsonResource";
import WorkspacePreview from "./WorkspacePreview";
import {
  clampWorkspaceSelection,
  filterWorkspaceRows,
  WORKSPACE_GATES,
  workspaceCounts,
  WORKSPACE_STAGES,
} from "./workspaceModel";
import type { LeafStatus, WorkspaceListResponse, WorkspaceStageFilter } from "./types";

function Progress({ status }: { status?: LeafStatus }) {
  const done = status?.progress_done || 0;
  const cur = status?.progress_current || 0;
  const label = status?.progress_label || "—/10";
  return (
    <span className="prog">
      <span className="segs">
        {WORKSPACE_GATES.map((n) => {
          const cls = n <= done ? "sg done" : n === cur ? "sg cur" : "sg";
          return <span key={n} className={cls} />;
        })}
      </span>
      <span className="pnum">{label}</span>
    </span>
  );
}

export default function WorkspaceList() {
  const { data, error } = useJsonResource<WorkspaceListResponse>("/api/list");
  const [stage, setStage] = useState<WorkspaceStageFilter>("all");
  const [q, setQ] = useState("");
  const [sel, setSel] = useState(0);
  const [previewOpen, setPreviewOpen] = useState(true);
  const filterRef = useRef<HTMLInputElement | null>(null);
  const rowRefs = useRef<Array<HTMLAnchorElement | null>>([]);

  const rows = useMemo(() => filterWorkspaceRows(data, { stage, query: q }), [data, stage, q]);

  const selectedIndex = clampWorkspaceSelection(sel, rows.length);
  const selectedRow = rows[selectedIndex];
  const openRow = useCallback(
    (index = selectedIndex) => {
      const it = rows[index];
      if (it) openLeaf(it.slug);
    },
    [rows, selectedIndex],
  );

  const isPlainPrimaryClick = (event: MouseEvent<HTMLAnchorElement>) =>
    event.button === 0 && !event.metaKey && !event.ctrlKey && !event.shiftKey && !event.altKey;

  const selectRow = (event: MouseEvent<HTMLAnchorElement>, index: number) => {
    if (!isPlainPrimaryClick(event)) return;
    event.preventDefault();
    setSel(index);
  };

  const openRowFromMouse = (event: MouseEvent<HTMLAnchorElement>, index: number) => {
    if (!isPlainPrimaryClick(event)) return;
    event.preventDefault();
    openRow(index);
  };

  useEffect(() => {
    const id = requestAnimationFrame(() => {
      const el = rowRefs.current[selectedIndex];
      if (!el) return;
      const rect = el.getBoundingClientRect();
      const topPad = 72;
      const bottomPad = 24;
      if (rect.top < topPad) {
        window.scrollBy({ top: rect.top - topPad, behavior: "smooth" });
      } else if (rect.bottom > window.innerHeight - bottomPad) {
        window.scrollBy({ top: rect.bottom - window.innerHeight + bottomPad, behavior: "smooth" });
      }
    });
    return () => cancelAnimationFrame(id);
  }, [selectedIndex, rows.length]);

  // keyboard: j/k move, g/G edges, Enter open, / focus filter, h/l stage, p preview
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (document.activeElement === filterRef.current) {
        if (e.key === "Escape") {
          setQ("");
          filterRef.current?.blur();
        }
        return;
      }
      if (e.key === "/") {
        e.preventDefault();
        filterRef.current?.focus();
      } else if (e.key === "j") {
        e.preventDefault();
        setSel((s) => (rows.length ? Math.min(rows.length - 1, s + 1) : 0));
      } else if (e.key === "k") {
        e.preventDefault();
        setSel((s) => Math.max(0, s - 1));
      } else if (e.key === "g") {
        e.preventDefault();
        setSel(0);
      } else if (e.key === "G") {
        e.preventDefault();
        setSel(rows.length ? rows.length - 1 : 0);
      } else if (e.key === "Enter") {
        openRow();
      } else if (e.key === "p") {
        e.preventDefault();
        setPreviewOpen((open) => !open);
      } else if (e.key === "d" || e.key === "u") {
        e.preventDefault();
        window.scrollBy({
          top: (e.key === "d" ? 1 : -1) * window.innerHeight * 0.85,
          behavior: "smooth",
        });
      } else if (e.key === "h" || e.key === "l") {
        const i = WORKSPACE_STAGES.indexOf(stage);
        const next = e.key === "l" ? Math.min(WORKSPACE_STAGES.length - 1, i + 1) : Math.max(0, i - 1);
        setStage(WORKSPACE_STAGES[next]);
        setSel(0);
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [openRow, rows.length, stage]);

  if (error) return <p className="err">목록을 불러오지 못했습니다: {error}. `leaf serve`가 떠 있나요?</p>;
  if (!data) return <p className="muted">불러오는 중…</p>;

  const counts = workspaceCounts(data);

  return (
    <div className="ws">
      <h1 className="vtitle">Workspace</h1>
      <p className="vsub">
        {counts.leaves || 0} leaves &middot; {counts.sprouts || 0} sprouts &middot; {counts.fallen || 0} fallen
      </p>

      <div className="tools">
        <div className="filter">
          <span className="hint">/</span>
          <input
            ref={filterRef}
            id="workspace-filter"
            name="workspace-filter"
            value={q}
            enterKeyHint="go"
            onChange={(e) => {
              setQ(e.target.value);
              setSel(0);
            }}
            onKeyDown={(e) => {
              if (e.key === "Enter") {
                e.preventDefault();
                openRow();
              }
            }}
            placeholder="filter by slug, action"
          />
        </div>
        <div className="stageseg">
          {WORKSPACE_STAGES.map((s) => (
            <button
              key={s}
              className={stage === s ? "on" : ""}
              onClick={() => {
                setStage(s);
                setSel(0);
              }}
            >
              {s}
              {s !== "all" && <span className="c"> {counts[s] || 0}</span>}
            </button>
          ))}
        </div>
        <button
          aria-pressed={previewOpen}
          className={`preview-toggle${previewOpen ? " on" : ""}`}
          onClick={() => setPreviewOpen((open) => !open)}
          title="Toggle workspace preview"
          type="button"
        >
          Preview <span className="kbd">p</span>
        </button>
      </div>

      <div className={`workspace-grid${previewOpen ? " with-preview" : ""}`}>
        <div className="tbl">
          <div className="thead">
            <span>Leaf</span>
            <span>Phase &#8250; Gate</span>
            <span>Progress</span>
            <span style={{ textAlign: "right" }}>Status</span>
          </div>
          {rows.map((it, i) => (
            <a
              key={it.slug}
              ref={(el) => {
                rowRefs.current[i] = el;
              }}
              className={`trow${i === selectedIndex ? " sel" : ""}`}
              href={leafHref(it.slug)}
              onClick={(event) => selectRow(event, i)}
              onDoubleClick={(event) => openRowFromMouse(event, i)}
            >
              <div className="c-leaf">
                <div className="slug">
                  <span className={`tagdot ${it._stage}`} />
                  {it.slug}
                </div>
                <div className="why">{it.status?.next_action || "—"}</div>
              </div>
              <div className="phase">
                <b>{it.status?.current_phase || "—"}</b> {it.status?.current_gate ? `› ${it.status.current_gate}` : ""}
              </div>
              <Progress status={it.status} />
              <div className={`status ${it.status?.parse_state || "ok"}`}>
                <span className="d" /> {it.status?.parse_state || "ok"}
              </div>
            </a>
          ))}
        </div>
        {previewOpen && selectedRow && <WorkspacePreview row={selectedRow} />}
      </div>
      <p className="foot-note">
        {rows.length} shown
        <span className="khint">
          <span className="kbd">j</span>
          <span className="kbd">k</span>
          <span className="kbd">g</span>
          <span className="kbd">G</span> 이동 &middot; <span className="kbd">Enter</span> 열기 &middot;{" "}
          <span className="kbd">d</span>
          <span className="kbd">u</span> 페이지 &middot; <span className="kbd">/</span> 필터 &middot;{" "}
          <span className="kbd">p</span> preview &middot; <span className="kbd">h</span>
          <span className="kbd">l</span> stage &middot; click 선택 &middot; double-click 열기
        </span>
      </p>
    </div>
  );
}
