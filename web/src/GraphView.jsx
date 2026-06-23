import React, { useEffect, useMemo, useState } from "react";

// Lightweight layout: place nodes on a circle (deterministic, no layout lib).
// Good enough for the pressed corpus scale; a force/dagre layout is a later slice.
function layout(nodes, w, h) {
  const cx = w / 2;
  const cy = h / 2;
  const r = Math.min(w, h) * 0.34;
  const pos = {};
  const n = nodes.length || 1;
  nodes.forEach((node, i) => {
    const a = (i / n) * Math.PI * 2 - Math.PI / 2;
    pos[node.id] = { x: cx + r * Math.cos(a), y: cy + r * Math.sin(a) };
  });
  return pos;
}

export default function GraphView() {
  const [data, setData] = useState(null);
  const [error, setError] = useState(null);
  const [sel, setSel] = useState(null);
  const W = 760;
  const H = 460;

  useEffect(() => {
    fetch("/api/graph")
      .then((r) => (r.ok ? r.json() : Promise.reject(new Error(`HTTP ${r.status}`))))
      .then((d) => {
        setData(d);
        setSel(d.nodes[0] || null);
      })
      .catch((e) => setError(e.message));
  }, []);

  const pos = useMemo(() => (data ? layout(data.nodes, W, H) : {}), [data]);

  if (error) return <p className="err">그래프를 불러오지 못했습니다: {error}</p>;
  if (!data) return <p className="muted">불러오는 중…</p>;

  return (
    <div className="graph">
      <h1 className="vtitle">Knowledge graph</h1>
      <p className="vsub">
        Pressed 디지트와 관계 &middot; nodes {data.nodes.length} · edges {data.edges.length}
      </p>
      <div className="gr">
        <div className="canvas">
          <svg viewBox={`0 0 ${W} ${H}`} width="100%" height={H}>
            <defs>
              <marker id="ar" markerWidth="9" markerHeight="9" refX="7" refY="4" orient="auto">
                <path d="M0,0 L8,4 L0,8 z" fill="#a8aab0" />
              </marker>
            </defs>
            {data.edges.map((e, i) => {
              const a = pos[e.source];
              const b = pos[e.target];
              if (!a || !b) {
                // dangling target (e.g. superseded → fallen leaf left the graph)
                return null;
              }
              const mx = (a.x + b.x) / 2;
              const my = (a.y + b.y) / 2;
              return (
                <g key={i}>
                  <line
                    x1={a.x}
                    y1={a.y}
                    x2={b.x}
                    y2={b.y}
                    stroke="#cfc9bc"
                    strokeWidth="1.6"
                    markerEnd="url(#ar)"
                  />
                  <text className="edge-lbl" x={mx} y={my - 6} textAnchor="middle">
                    {e.predicate}
                  </text>
                </g>
              );
            })}
            {data.nodes.map((node) => {
              const p = pos[node.id];
              const isSel = sel && sel.id === node.id;
              return (
                <g
                  key={node.id}
                  transform={`translate(${p.x},${p.y})`}
                  className={`node${isSel ? " sel" : ""}`}
                  onClick={() => setSel(node)}
                  onDoubleClick={() => (window.location.hash = `#/leaf/${node.slug}`)}
                  style={{ cursor: "pointer" }}
                >
                  <circle r="7" fill="#b5862a" />
                  <text className="node-t" y="24" textAnchor="middle">
                    {node.title.length > 22 ? node.title.slice(0, 21) + "…" : node.title}
                  </text>
                </g>
              );
            })}
          </svg>
        </div>
        <aside className="gpanel">
          {sel ? (
            <>
              <h3>{sel.title}</h3>
              <div className="gh">{sel.id}</div>
              <p>{sel.description}</p>
              <div className="tags">
                {(sel.tags || []).map((t) => (
                  <span key={t} className="tag">
                    #{t}
                  </span>
                ))}
              </div>
              <a className="btn" href={`#/leaf/${sel.slug}`}>
                본문 열기 → Leaf detail
              </a>
            </>
          ) : (
            <p className="muted">노드를 선택하세요.</p>
          )}
          <p className="gnote">노드 더블클릭 → 리뷰 리더. 단글링(fallen) 타깃 엣지는 숨김(03-fallen 추적).</p>
        </aside>
      </div>
    </div>
  );
}
