import React from "react";
import type { RefObject } from "react";
import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";
import { REVIEW_REF_FOCUS, progressWidth, referenceCount } from "./reviewReaderModel";
import { leafHref } from "./routes";
import type { ReviewRefFocus, ReviewReference, ReviewSource } from "./types";

const MARKDOWN_PLUGINS = [remarkGfm];

export const MarkdownContent = React.memo(function MarkdownContent({ children }: { children: string }) {
  return <ReactMarkdown remarkPlugins={MARKDOWN_PLUGINS}>{children}</ReactMarkdown>;
});

export function GateNav({
  sources,
  active,
  onSelect,
}: {
  sources: ReviewSource[];
  active: number;
  onSelect: (index: number) => void;
}) {
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

export function MobileReaderActions({
  activeGate,
  progress,
  references,
  onOpenToc,
  onOpenReferences,
}: {
  activeGate?: string;
  progress: number;
  references: ReviewReference[];
  onOpenToc: () => void;
  onOpenReferences: () => void;
}) {
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

export function TocOverlay({
  sources,
  active,
  onSelect,
  onClose,
}: {
  sources: ReviewSource[];
  active: number;
  onSelect: (index: number) => void;
  onClose: () => void;
}) {
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

export function ReferencesDrawer({
  references,
  refFocus,
  selectedIndex,
  readRef,
  onClose,
  onOpenFullPage,
  onSelectReference,
}: {
  references: ReviewReference[];
  refFocus: ReviewRefFocus;
  selectedIndex: number;
  readRef: RefObject<HTMLDivElement>;
  onClose: () => void;
  onOpenFullPage: () => void;
  onSelectReference: (index: number) => void;
}) {
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
                <span className="kbd">f</span> 전체 &middot; <span className="kbd">h</span> 닫기
              </>
            ) : (
              <>
                <span className="kbd">j</span>
                <span className="kbd">k</span>
                <span className="kbd">d</span>
                <span className="kbd">u</span> 스크롤 &middot; <span className="kbd">f</span> 전체 &middot;{" "}
                <span className="kbd">h</span> 목록
              </>
            )}
          </span>
          <span className="refactions">
            <button
              type="button"
              className="reficon"
              disabled={!selected}
              title="전체 페이지로 보기 (f)"
              aria-label="전체 페이지로 보기"
              aria-keyshortcuts="F"
              onClick={onOpenFullPage}
            >
              ⛶
            </button>
            <button className="refclose" onClick={onClose}>
              ✕
            </button>
          </span>
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

export function ReferenceFullPage({
  reference,
  referencePath,
  slug,
}: {
  reference?: ReviewReference;
  referencePath: string;
  slug: string;
}) {
  return (
    <>
      <p className="crumb">
        <a href={leafHref(slug)}>← review로 돌아가기</a> &nbsp;·&nbsp; reference &middot;{" "}
        <b>{referencePath.split("/").pop()}</b>
        <span className="khint">
          <span className="kbd">j</span>
          <span className="kbd">k</span> 스크롤 &middot; <span className="kbd">h</span>
          <span className="kbd">l</span> reference &middot; <span className="kbd">q</span> review
        </span>
      </p>
      <article className="report reference-full">
        <div className="file">{referencePath}</div>
        {reference ? (
          <div className="md">
            <MarkdownContent>{reference.markdown}</MarkdownContent>
          </div>
        ) : (
          <p className="err">레퍼런스를 찾지 못했습니다: {referencePath}</p>
        )}
      </article>
    </>
  );
}
