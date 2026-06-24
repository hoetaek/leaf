import React from "react";
import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";
import { REVIEW_REF_FOCUS, progressWidth, referenceCount } from "./reviewReaderModel.js";

const MARKDOWN_PLUGINS = [remarkGfm];

export const MarkdownContent = React.memo(function MarkdownContent({ children }) {
  return <ReactMarkdown remarkPlugins={MARKDOWN_PLUGINS}>{children}</ReactMarkdown>;
});

export function GateNav({ sources, active, onSelect }) {
  return (
    <nav className="gnav">
      {sources.map((source, index) => (
        <a
          key={index}
          className={index === active ? "on" : source.present ? "" : "empty"}
          onClick={() => onSelect(index)}
        >
          {source.gate}
          <span>{source.present ? "✓" : "·"}</span>
        </a>
      ))}
    </nav>
  );
}

export function MobileReaderActions({ activeGate, progress, references, onOpenToc, onOpenReferences }) {
  const count = references.length;

  return (
    <div className="mobile-reader-actions">
      <button className="toc-bar" onClick={onOpenToc}>
        <span className="toc-bar-gate">{activeGate || "목차"}</span>
        <span className="toc-bar-action">목차</span>
        <i style={{ width: progressWidth(progress) }} />
      </button>
      {count > 0 && (
        <button className="refs-bar" onClick={onOpenReferences}>
          Refs <span>{count}</span>
        </button>
      )}
    </div>
  );
}

export function TocOverlay({ sources, active, onSelect, onClose }) {
  return (
    <div className="toc-overlay" onClick={onClose}>
      <div className="toc-sheet" onClick={(event) => event.stopPropagation()}>
        <div className="toc-sheethead">
          <b>Gates</b>
          <button className="refclose" onClick={onClose}>
            ✕
          </button>
        </div>
        <GateNav
          sources={sources}
          active={active}
          onSelect={(index) => {
            onSelect(index);
            onClose();
          }}
        />
      </div>
    </div>
  );
}

export function ReferencesDrawer({ references, refFocus, selectedIndex, readRef, onClose, onSelectReference }) {
  const selected = references[selectedIndex];

  return (
    <div className="refoverlay" onClick={onClose}>
      <aside className="refdrawer" onClick={(event) => event.stopPropagation()}>
        <div className="refhead">
          <b>References</b> <span className="muted">({referenceCount({ references })})</span>
          <span className="khint">
            {refFocus === REVIEW_REF_FOCUS.LIST ? (
              <>
                <span className="kbd">j</span>
                <span className="kbd">k</span> 이동 &middot; <span className="kbd">l</span> 선택 &middot;{" "}
                <span className="kbd">h</span> 닫기
              </>
            ) : (
              <>
                <span className="kbd">j</span>
                <span className="kbd">k</span>
                <span className="kbd">d</span>
                <span className="kbd">u</span> 스크롤 &middot; <span className="kbd">h</span> 목록
              </>
            )}
          </span>
          <button className="refclose" onClick={onClose}>
            ✕
          </button>
        </div>
        {references.length === 0 ? (
          <p className="muted">이 leaf에는 레퍼런스가 없습니다.</p>
        ) : (
          <div className="refpicker">
            <ul className={`reflist-nav${refFocus === REVIEW_REF_FOCUS.LIST ? " focus" : ""}`}>
              {references.map((reference, index) => (
                <li
                  key={index}
                  className={index === selectedIndex ? "on" : ""}
                  onClick={() => onSelectReference(index)}
                >
                  {reference.relative_path.split("/").pop()}
                </li>
              ))}
            </ul>
            <div ref={readRef} className={`refread md${refFocus === REVIEW_REF_FOCUS.CONTENT ? " focus" : ""}`}>
              <div className="file">{selected?.relative_path}</div>
              <MarkdownContent>{selected?.markdown || ""}</MarkdownContent>
            </div>
          </div>
        )}
      </aside>
    </div>
  );
}
