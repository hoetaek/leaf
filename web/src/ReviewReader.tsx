import { useCallback, useMemo, useRef, useState } from "react";
import {
  GateNav,
  MarkdownContent,
  MobileReaderActions,
  ReferenceFullPage,
  ReferencesDrawer,
  TocOverlay,
} from "./ReviewReaderParts";
import { nextReferenceIndex, progressWidth, reviewResourcePath, REVIEW_REF_FOCUS } from "./reviewReaderModel";
import { openReference } from "./routes";
import { useActiveReviewSection } from "./useActiveReviewSection";
import { useJsonResource } from "./useJsonResource";
import { useReadingProgress } from "./useReadingProgress";
import { useReviewKeyboardShortcuts } from "./useReviewKeyboardShortcuts";
import type { ReviewRefFocus, ReviewResponse } from "./types";

interface ReviewReaderProps {
  referencePath?: string;
  slug: string;
}

export default function ReviewReader({ referencePath, slug }: ReviewReaderProps) {
  const { data, error } = useJsonResource<ReviewResponse>(reviewResourcePath(slug));
  const [showRefs, setShowRefs] = useState(false);
  const [refSel, setRefSel] = useState(0);
  const [refFocus, setRefFocus] = useState<ReviewRefFocus>(REVIEW_REF_FOCUS.LIST);
  const [showToc, setShowToc] = useState(false);
  const reportRef = useRef<HTMLElement | null>(null);
  const refReadRef = useRef<HTMLDivElement | null>(null);
  const { active, sectionRefs, jump } = useActiveReviewSection(data);
  const progress = useReadingProgress(data, reportRef);
  const references = useMemo(() => data?.references || [], [data?.references]);
  const selectedReference = references[refSel];
  const fullPageReferenceIndex = referencePath
    ? references.findIndex((reference) => reference.relative_path === referencePath)
    : -1;
  const openSelectedReference = useCallback(() => {
    if (!data || !selectedReference) return;
    openReference(data.slug, selectedReference.relative_path);
  }, [data, selectedReference]);
  const openFullPageReferenceByStep = useCallback(
    (step: number) => {
      if (!data || fullPageReferenceIndex < 0) return;

      const nextIndex = nextReferenceIndex(fullPageReferenceIndex, step, references.length);
      if (nextIndex === fullPageReferenceIndex) return;

      const nextReference = references[nextIndex];
      if (nextReference) openReference(data.slug, nextReference.relative_path);
    },
    [data, fullPageReferenceIndex, references],
  );
  const openPreviousFullPageReference = useCallback(
    () => openFullPageReferenceByStep(-1),
    [openFullPageReferenceByStep],
  );
  const openNextFullPageReference = useCallback(() => openFullPageReferenceByStep(1), [openFullPageReferenceByStep]);

  useReviewKeyboardShortcuts({
    data,
    onNextReference: referencePath ? openNextFullPageReference : undefined,
    onOpenReferenceFullPage: openSelectedReference,
    onPreviousReference: referencePath ? openPreviousFullPageReference : undefined,
    refFocus,
    refReadRef,
    setRefFocus,
    setRefSel,
    setShowRefs,
    showRefs,
  });

  if (!slug) {
    return <p className="err">리뷰 slug가 없습니다.</p>;
  }
  if (error) {
    return <p className="err">리뷰를 불러오지 못했습니다: {error}. `leaf serve`가 떠 있나요?</p>;
  }
  if (!data) {
    return <p className="muted">불러오는 중…</p>;
  }

  const openReferences = () => {
    setShowRefs(true);
    setRefSel(0);
    setRefFocus(REVIEW_REF_FOCUS.CONTENT);
  };
  const selectReference = (index: number) => {
    setRefSel(index);
    setRefFocus(REVIEW_REF_FOCUS.CONTENT);
  };
  const fullPageReference = referencePath
    ? references.find((reference) => reference.relative_path === referencePath)
    : undefined;

  if (referencePath) {
    return <ReferenceFullPage reference={fullPageReference} referencePath={referencePath} slug={data.slug} />;
  }

  return (
    <>
      <p className="crumb">
        <a href="#/">← workspace</a> &nbsp;·&nbsp; review &middot; <b>{data.slug}</b>
        <span className="khint">
          <span className="kbd">j</span>
          <span className="kbd">k</span> 줄 &middot; <span className="kbd">d</span>
          <span className="kbd">u</span> 페이지 &middot; <span className="kbd">R</span> references (
          {data.references?.length || 0}) &middot; <span className="kbd">q</span> 목록
        </span>
      </p>
      <MobileReaderActions
        activeGate={data.sources[active]?.gate}
        progress={progress}
        references={references}
        onOpenToc={() => setShowToc(true)}
        onOpenReferences={openReferences}
      />
      <div className="reader">
        <aside className="rail">
          <h4>Gates &mdash; read all</h4>
          <GateNav sources={data.sources} active={active} onSelect={jump} />
        </aside>

        <article className="report" ref={reportRef}>
          <div className="rprog">
            <i style={{ width: progressWidth(progress) }} />
          </div>
          {data.sources.map((s, i) => (
            <section
              key={i}
              data-idx={i}
              ref={(el) => {
                sectionRefs.current[i] = el;
              }}
            >
              <div className="phtag">{s.phase}</div>
              <div className="file">
                {s.gate} &nbsp;&middot;&nbsp; {s.relative_path}
              </div>
              {s.present ? (
                <div className="md">
                  <MarkdownContent>{s.markdown}</MarkdownContent>
                </div>
              ) : (
                <p className="muted">(이 게이트 문서는 아직 없음)</p>
              )}
            </section>
          ))}
        </article>
      </div>

      {/* mobile: collapsed TOC (desktop uses the sticky rail) */}
      {showToc && (
        <TocOverlay sources={data.sources} active={active} onSelect={jump} onClose={() => setShowToc(false)} />
      )}

      {showRefs && (
        <ReferencesDrawer
          references={references}
          refFocus={refFocus}
          selectedIndex={refSel}
          readRef={refReadRef}
          onClose={() => setShowRefs(false)}
          onOpenFullPage={openSelectedReference}
          onSelectReference={selectReference}
        />
      )}
    </>
  );
}
