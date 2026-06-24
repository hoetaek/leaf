import { useEffect } from "react";
import { isTextEntryElement, nextReferenceIndex, referenceCount, REVIEW_REF_FOCUS } from "./reviewReaderModel.js";

export function useReviewKeyboardShortcuts({
  data,
  refFocus,
  refReadRef,
  setRefFocus,
  setRefSel,
  setShowRefs,
  showRefs,
}) {
  useEffect(() => {
    const count = referenceCount(data);
    const inContent = showRefs && refFocus === REVIEW_REF_FOCUS.CONTENT;
    const pane = () => (inContent ? refReadRef.current : null);
    const scroll = (dy) => (pane() || window).scrollBy({ top: dy, behavior: "smooth" });
    const page = (frac) => {
      const element = pane();
      scroll(frac * (element ? element.clientHeight : window.innerHeight));
    };
    const onKey = (event) => {
      if (isTextEntryElement(document.activeElement)) return;

      switch (event.key) {
        case "q":
        case "Escape":
          if (showRefs) setShowRefs(false);
          else window.location.hash = "#/";
          break;
        case "r":
        case "R":
          event.preventDefault();
          setShowRefs((current) => !current);
          setRefSel(0);
          setRefFocus(REVIEW_REF_FOCUS.LIST);
          break;
        case "l":
        case "ArrowRight":
          if (showRefs && refFocus === REVIEW_REF_FOCUS.LIST && count > 0) {
            event.preventDefault();
            setRefFocus(REVIEW_REF_FOCUS.CONTENT);
          }
          break;
        case "h":
        case "ArrowLeft":
          if (showRefs) {
            event.preventDefault();
            if (refFocus === REVIEW_REF_FOCUS.CONTENT) setRefFocus(REVIEW_REF_FOCUS.LIST);
            else setShowRefs(false);
          }
          break;
        case "j":
        case "ArrowDown":
          event.preventDefault();
          if (showRefs && refFocus === REVIEW_REF_FOCUS.LIST) {
            setRefSel((current) => nextReferenceIndex(current, 1, count));
          } else {
            scroll(90);
          }
          break;
        case "k":
        case "ArrowUp":
          event.preventDefault();
          if (showRefs && refFocus === REVIEW_REF_FOCUS.LIST) {
            setRefSel((current) => nextReferenceIndex(current, -1, count));
          } else {
            scroll(-90);
          }
          break;
        case "d":
          event.preventDefault();
          page(0.85);
          break;
        case "u":
          event.preventDefault();
          page(-0.85);
          break;
        default:
          break;
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [data, refFocus, refReadRef, setRefFocus, setRefSel, setShowRefs, showRefs]);
}
