import React from "react";
import { leafHref } from "./routes.js";

export default function GraphDetailsPanel({ selected, hiddenEdgeCount }) {
  return (
    <aside className="gpanel">
      {selected ? (
        <>
          <h3>{selected.title}</h3>
          <div className="gh">{selected.id}</div>
          <div className="gstats">
            <span>degree {selected.degree}</span>
            <span>tags {selected.tags.length}</span>
          </div>
          <p>{selected.description || "설명이 없습니다."}</p>
          <div className="tags">
            {selected.tags.map((tag) => (
              <span key={tag} className="tag">
                #{tag}
              </span>
            ))}
          </div>
          <a className="btn" href={leafHref(selected.slug)}>
            본문 열기 → Leaf detail
          </a>
        </>
      ) : (
        <p className="muted">노드를 선택하세요.</p>
      )}
      {hiddenEdgeCount ? <p className="gnote">현재 graph에 없는 fallen 타깃 edge {hiddenEdgeCount}개는 숨겼습니다.</p> : null}
    </aside>
  );
}
