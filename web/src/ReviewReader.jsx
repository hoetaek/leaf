import React, { useEffect, useRef, useState } from "react";
import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";

// react-markdown needs remark-gfm for GFM tables / strikethrough / task lists.
function Md({ children }) {
  return <ReactMarkdown remarkPlugins={[remarkGfm]}>{children}</ReactMarkdown>;
}

export default function ReviewReader({ slug = "leaf-tend-pressed-docs" }) {
  const [data, setData] = useState(null);
  const [error, setError] = useState(null);
  const [active, setActive] = useState(0);
  const [progress, setProgress] = useState(0);
  const [showRefs, setShowRefs] = useState(false);
  const [refSel, setRefSel] = useState(0);
  const [refFocus, setRefFocus] = useState("list"); // "list" | "content"
  const [showToc, setShowToc] = useState(false); // mobile floating table-of-contents
  const sectionRefs = useRef([]);
  const reportRef = useRef(null);

  // References picker behaves like a LazyVim two-pane explorer:
  //  drawer + list focus  : j/k move the reference list · l selects (focus → content)
  //  drawer + content focus: j/k line-scroll · d/u page-scroll · h back to the list
  //  q / Esc -> close drawer (if open) else back to workspace
  //  drawer CLOSED        : j/k line-scroll · d/u page-scroll the whole review
  useEffect(() => {
    const n = data?.references?.length || 0;
    const inContent = showRefs && refFocus === "content";
    const pane = () => (inContent ? document.querySelector(".refread") : null);
    const scroll = (dy) => (pane() || window).scrollBy({ top: dy, behavior: "smooth" });
    const page = (frac) => {
      const el = pane();
      scroll(frac * (el ? el.clientHeight : window.innerHeight));
    };
    const onKey = (e) => {
      const tag = document.activeElement?.tagName;
      if (tag === "INPUT" || tag === "TEXTAREA") return;
      switch (e.key) {
        case "q":
        case "Escape":
          if (showRefs) setShowRefs(false);
          else window.location.hash = "#/";
          break;
        case "r":
        case "R":
          e.preventDefault();
          setShowRefs((s) => !s);
          setRefSel(0);
          setRefFocus("list");
          break;
        case "l":
        case "ArrowRight":
          if (showRefs && refFocus === "list" && n > 0) {
            e.preventDefault();
            setRefFocus("content");
          }
          break;
        case "h":
        case "ArrowLeft":
          if (showRefs) {
            e.preventDefault();
            if (refFocus === "content") setRefFocus("list");
            else setShowRefs(false);
          }
          break;
        case "j":
        case "ArrowDown":
          e.preventDefault();
          if (showRefs && refFocus === "list") setRefSel((s) => Math.min(n - 1, s + 1));
          else scroll(90);
          break;
        case "k":
        case "ArrowUp":
          e.preventDefault();
          if (showRefs && refFocus === "list") setRefSel((s) => Math.max(0, s - 1));
          else scroll(-90);
          break;
        case "d":
          e.preventDefault();
          page(0.85);
          break;
        case "u":
          e.preventDefault();
          page(-0.85);
          break;
        default:
          break;
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [showRefs, refFocus, data]);

  useEffect(() => {
    let alive = true;
    fetch(`/api/review/${slug}`)
      .then((r) => (r.ok ? r.json() : Promise.reject(new Error(`HTTP ${r.status}`))))
      .then((d) => alive && setData(d))
      .catch((e) => alive && setError(e.message));
    return () => {
      alive = false;
    };
  }, [slug]);

  // scrollspy: highlight the gate whose section is in view
  useEffect(() => {
    if (!data) return;
    const io = new IntersectionObserver(
      (entries) => {
        entries.forEach((e) => {
          if (e.isIntersecting) {
            const i = Number(e.target.dataset.idx);
            if (!Number.isNaN(i)) setActive(i);
          }
        });
      },
      { rootMargin: "-64px 0px -70% 0px", threshold: 0 },
    );
    sectionRefs.current.forEach((s) => s && io.observe(s));
    return () => io.disconnect();
  }, [data]);

  // reading progress
  useEffect(() => {
    const onScroll = () => {
      const rep = reportRef.current;
      if (!rep) return;
      const r = rep.getBoundingClientRect();
      const total = r.height - window.innerHeight;
      setProgress(Math.min(1, Math.max(0, -r.top / (total || 1))));
    };
    window.addEventListener("scroll", onScroll, { passive: true });
    onScroll();
    return () => window.removeEventListener("scroll", onScroll);
  }, [data]);

  if (error) {
    return <p className="err">리뷰를 불러오지 못했습니다: {error}. `leaf serve`가 떠 있나요?</p>;
  }
  if (!data) {
    return <p className="muted">불러오는 중…</p>;
  }

  const jump = (i) =>
    sectionRefs.current[i]?.scrollIntoView({ behavior: "smooth", block: "start" });

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
      <button className="toc-bar" onClick={() => setShowToc(true)}>
        <span className="toc-bar-gate">{data.sources[active]?.gate || "목차"}</span>
        <span className="toc-bar-action">목차</span>
        <i style={{ width: `${(progress * 100).toFixed(1)}%` }} />
      </button>
      <div className="reader">
        <aside className="rail">
          <h4>Gates &mdash; read all</h4>
          <nav className="gnav">
            {data.sources.map((s, i) => (
              <a
                key={i}
                className={i === active ? "on" : s.present ? "" : "empty"}
                onClick={() => jump(i)}
              >
                {gateLabel(s.gate)}
                <span>{s.present ? "✓" : "·"}</span>
              </a>
            ))}
          </nav>
        </aside>

        <article className="report" ref={reportRef}>
          <div className="rprog">
            <i style={{ width: `${(progress * 100).toFixed(1)}%` }} />
          </div>
          {data.sources.map((s, i) => (
            <section
              key={i}
              data-idx={i}
              ref={(el) => (sectionRefs.current[i] = el)}
            >
              <div className="phtag">{s.phase}</div>
              <div className="file">
                {s.gate} &nbsp;&middot;&nbsp; {s.relative_path}
              </div>
              {s.present ? (
                <div className="md">
                  <Md>{s.markdown}</Md>
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
        <div className="toc-overlay" onClick={() => setShowToc(false)}>
          <div className="toc-sheet" onClick={(e) => e.stopPropagation()}>
            <div className="toc-sheethead">
              <b>Gates</b>
              <button className="refclose" onClick={() => setShowToc(false)}>
                ✕
              </button>
            </div>
            <nav className="gnav">
              {data.sources.map((s, i) => (
                <a
                  key={i}
                  className={i === active ? "on" : s.present ? "" : "empty"}
                  onClick={() => {
                    jump(i);
                    setShowToc(false);
                  }}
                >
                  {s.gate}
                  <span>{s.present ? "✓" : "·"}</span>
                </a>
              ))}
            </nav>
          </div>
        </div>
      )}

      {showRefs && (
        <div className="refoverlay" onClick={() => setShowRefs(false)}>
          <aside className="refdrawer" onClick={(e) => e.stopPropagation()}>
            <div className="refhead">
              <b>References</b>{" "}
              <span className="muted">({data.references?.length || 0})</span>
              <span className="khint">
                {refFocus === "list" ? (
                  <>
                    <span className="kbd">j</span>
                    <span className="kbd">k</span> 이동 &middot; <span className="kbd">l</span> 선택
                    &middot; <span className="kbd">h</span> 닫기
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
              <button className="refclose" onClick={() => setShowRefs(false)}>
                ✕
              </button>
            </div>
            {(data.references?.length || 0) === 0 ? (
              <p className="muted">이 leaf에는 레퍼런스가 없습니다.</p>
            ) : (
              <div className="refpicker">
                <ul className={`reflist-nav${refFocus === "list" ? " focus" : ""}`}>
                  {data.references.map((r, i) => (
                    <li
                      key={i}
                      className={i === refSel ? "on" : ""}
                      onClick={() => {
                        setRefSel(i);
                        setRefFocus("content");
                      }}
                    >
                      {r.relative_path.split("/").pop()}
                    </li>
                  ))}
                </ul>
                <div className={`refread md${refFocus === "content" ? " focus" : ""}`}>
                  <div className="file">{data.references[refSel]?.relative_path}</div>
                  <Md>{data.references[refSel]?.markdown || ""}</Md>
                </div>
              </div>
            )}
          </aside>
        </div>
      )}
    </>
  );
}

// "① Intent" -> shows as-is; keep the gate string from the server.
function gateLabel(gate) {
  return gate;
}
