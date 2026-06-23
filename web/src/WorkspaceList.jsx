import React, { useEffect, useMemo, useRef, useState } from "react";

const GLYPH = ["①", "②", "③", "④", "⑤", "⑥", "⑦", "⑧", "⑨", "⑩"];
const GATE_INDEX = Object.fromEntries(GLYPH.map((g, i) => [g, i + 1]));

// gate number from a "③ Criteria" style string -> 1..10 (0 if none)
function gateNum(gate) {
  if (!gate) return 0;
  const g = gate.trim()[0];
  return GATE_INDEX[g] || 0;
}

function Progress({ phase, gate }) {
  const text = `${phase || ""} ${gate || ""}`;
  // A finished leaf (completed / pressed / leaf-done) reads as 10/10 even though
  // its gate string is no longer a ①–⑩ glyph.
  const finished = /완료|pressed|leaf-done|done|complete/i.test(text);
  const cur = finished ? 0 : gateNum(gate);
  const done = finished ? 10 : Math.max(0, cur - 1);
  const label = finished ? "10/10" : cur ? `${cur}/10` : "—/10";
  return (
    <span className="prog">
      <span className="segs">
        {GLYPH.map((_, i) => {
          const n = i + 1;
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

  useEffect(() => {
    fetch("/api/list")
      .then((r) => (r.ok ? r.json() : Promise.reject(new Error(`HTTP ${r.status}`))))
      .then(setData)
      .catch((e) => setError(e.message));
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

  // keyboard: j/k move, Enter open, / focus filter, h/l stage
  const stages = ["all", "sprouts", "leaves", "fallen"];
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
        const it = rows[sel];
        if (it) window.location.hash = `#/leaf/${encodeURIComponent(it.slug)}`;
      } else if (e.key === "h" || e.key === "l") {
        const i = stages.indexOf(stage);
        const next = e.key === "l" ? Math.min(stages.length - 1, i + 1) : Math.max(0, i - 1);
        setStage(stages[next]);
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
            onChange={(e) => {
              setQ(e.target.value);
              setSel(0);
            }}
            placeholder="filter by slug, action"
          />
        </div>
        <div className="stageseg">
          {stages.map((s) => (
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
            className={`trow${i === sel ? " sel" : ""}`}
            href={`#/leaf/${encodeURIComponent(it.slug)}`}
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
            <Progress phase={it.status?.current_phase} gate={it.status?.current_gate} />
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
          <span className="kbd">/</span> 필터 &middot; <span className="kbd">h</span>
          <span className="kbd">l</span> stage
        </span>
      </p>
    </div>
  );
}
