import React, { useEffect, useMemo, useRef, useState } from "react";
import { fetchJson } from "./api.js";
import { leafHref, openLeaf } from "./routes.js";

const GATES = Array.from({ length: 10 }, (_, i) => i + 1);
const STAGES = ["all", "sprouts", "leaves", "fallen"];

function Progress({ status }) {
  const done = status?.progress_done || 0;
  const cur = status?.progress_current || 0;
  const label = status?.progress_label || "—/10";
  return (
    <span className="prog">
      <span className="segs">
        {GATES.map((n) => {
          const cls = n <= done ? "sg done" : n === cur ? "sg cur" : "sg";
          return <span key={n} className={cls} />;
        })}
      </span>
      <span className="pnum">{label}</span>
    </span>
  );
}

export default function WorkspaceList() {
  const [data, setData] = useState(null);
  const [error, setError] = useState(null);
  const [stage, setStage] = useState("all");
  const [q, setQ] = useState("");
  const [sel, setSel] = useState(0);
  const filterRef = useRef(null);
  const rowRefs = useRef([]);

  useEffect(() => {
    fetchJson("/api/list").then(setData).catch((e) => setError(e.message));
  }, []);

  const rows = useMemo(() => {
    if (!data) return [];
    const all = [];
    for (const [key, st] of Object.entries(data.stages)) {
      for (const it of st.items) all.push({ ...it, _stage: key });
    }
    const query = q.trim().toLowerCase();
    return all.filter((it) => {
      const okStage = stage === "all" || it._stage === stage;
      const hay = `${it.slug} ${it.status?.next_action || ""}`.toLowerCase();
      return okStage && (!query || hay.includes(query));
    });
  }, [data, stage, q]);

  const openRow = (index = sel) => {
    const it = rows[index];
    if (it) openLeaf(it.slug);
  };

  useEffect(() => {
    if (rows.length === 0) {
      setSel(0);
      return;
    }
    setSel((s) => Math.min(s, rows.length - 1));
  }, [rows.length]);

  useEffect(() => {
    const id = requestAnimationFrame(() => {
      const el = rowRefs.current[sel];
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
  }, [sel, rows.length]);

  // keyboard: j/k move, Enter open, / focus filter, h/l stage
  useEffect(() => {
    const onKey = (e) => {
      if (document.activeElement === filterRef.current) {
        if (e.key === "Escape") {
          setQ("");
          filterRef.current.blur();
        }
        return;
      }
      if (e.key === "/") {
        e.preventDefault();
        filterRef.current?.focus();
      } else if (e.key === "j") {
        e.preventDefault();
        setSel((s) => Math.min(rows.length - 1, s + 1));
      } else if (e.key === "k") {
        e.preventDefault();
        setSel((s) => Math.max(0, s - 1));
      } else if (e.key === "Enter") {
        openRow();
      } else if (e.key === "d" || e.key === "u") {
        e.preventDefault();
        window.scrollBy({
          top: (e.key === "d" ? 1 : -1) * window.innerHeight * 0.85,
          behavior: "smooth",
        });
      } else if (e.key === "h" || e.key === "l") {
        const i = STAGES.indexOf(stage);
        const next = e.key === "l" ? Math.min(STAGES.length - 1, i + 1) : Math.max(0, i - 1);
        setStage(STAGES[next]);
        setSel(0);
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [rows, sel, stage]);

  if (error)
    return <p className="err">목록을 불러오지 못했습니다: {error}. `leaf serve`가 떠 있나요?</p>;
  if (!data) return <p className="muted">불러오는 중…</p>;

  const counts = Object.fromEntries(
    Object.entries(data.stages).map(([k, v]) => [k, v.count]),
  );

  return (
    <div className="ws">
      <h1 className="vtitle">Workspace</h1>
      <p className="vsub">
        {counts.leaves || 0} leaves &middot; {counts.sprouts || 0} sprouts &middot;{" "}
        {counts.fallen || 0} fallen
      </p>

      <div className="tools">
        <div className="filter">
          <span className="hint">/</span>
          <input
            ref={filterRef}
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
          {STAGES.map((s) => (
            <button key={s} className={stage === s ? "on" : ""} onClick={() => setStage(s)}>
              {s}
              {s !== "all" && <span className="c"> {counts[s] || 0}</span>}
            </button>
          ))}
        </div>
      </div>

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
            ref={(el) => (rowRefs.current[i] = el)}
            className={`trow${i === sel ? " sel" : ""}`}
            href={leafHref(it.slug)}
            onMouseEnter={() => setSel(i)}
          >
            <div className="c-leaf">
              <div className="slug">
                <span className={`tagdot ${it._stage}`} />
                {it.slug}
              </div>
              <div className="why">{it.status?.next_action || "—"}</div>
            </div>
            <div className="phase">
              <b>{it.status?.current_phase || "—"}</b>{" "}
              {it.status?.current_gate ? `› ${it.status.current_gate}` : ""}
            </div>
            <Progress status={it.status} />
            <div className={`status ${it.status?.parse_state || "ok"}`}>
              <span className="d" /> {it.status?.parse_state || "ok"}
            </div>
          </a>
        ))}
      </div>
      <p className="foot-note">
        {rows.length} shown
        <span className="khint">
          <span className="kbd">j</span>
          <span className="kbd">k</span> 이동 &middot; <span className="kbd">Enter</span> 열기 &middot;{" "}
          <span className="kbd">d</span>
          <span className="kbd">u</span> 페이지 &middot; <span className="kbd">/</span> 필터 &middot; <span className="kbd">h</span>
          <span className="kbd">l</span> stage
        </span>
      </p>
    </div>
  );
}
