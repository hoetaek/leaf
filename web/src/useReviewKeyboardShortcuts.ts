import { useEffect } from "react";
import type { Dispatch, RefObject, SetStateAction } from "react";
import { copyLeafCitation } from "./clipboard";
import { isTextEntryElement, nextReferenceIndex, referenceCount, REVIEW_REF_FOCUS } from "./reviewReaderModel";
import type { ReviewRefFocus, ReviewResponse } from "./types";

interface ReviewKeyboardShortcutOptions {
  data: ReviewResponse | null;
  onBack?: () => void;
  onNextReference?: () => void;
  onOpenReferenceFullPage?: () => void;
  onPreviousReference?: () => void;
  refFocus: ReviewRefFocus;
  refReadRef: RefObject<HTMLDivElement>;
  setRefFocus: Dispatch<SetStateAction<ReviewRefFocus>>;
  setRefSel: Dispatch<SetStateAction<number>>;
  setShowRefs: Dispatch<SetStateAction<boolean>>;
  showRefs: boolean;
}

export function useReviewKeyboardShortcuts({
  data,
  onBack,
  onNextReference,
  onOpenReferenceFullPage,
  onPreviousReference,
  refFocus,
  refReadRef,
  setRefFocus,
  setRefSel,
  setShowRefs,
  showRefs,
}: ReviewKeyboardShortcutOptions) {
  useEffect(() => {
    const count = referenceCount(data);
    const inContent = showRefs && refFocus === REVIEW_REF_FOCUS.CONTENT;
    const pane = () => (inContent ? refReadRef.current : null);
    const scroll = (dy: number) => (pane() || window).scrollBy({ top: dy, behavior: "smooth" });
    const scrollEdge = (bottom: boolean) => {
      const element = pane();
      if (element) {
        element.scrollTo({ top: bottom ? element.scrollHeight : 0, behavior: "smooth" });
      } else {
        window.scrollTo({ top: bottom ? document.documentElement.scrollHeight : 0, behavior: "smooth" });
      }
    };
    const page = (frac: number) => {
      const element = pane();
      scroll(frac * (element ? element.clientHeight : window.innerHeight));
    };
    const onKey = (event: KeyboardEvent) => {
      if (isTextEntryElement(document.activeElement)) return;
      if (event.metaKey || event.ctrlKey || event.altKey) return;

      switch (event.key) {
        case "q":
        case "Escape":
          if (showRefs) setShowRefs(false);
          else if (onBack) onBack();
          else window.location.hash = "#/";
          break;
        case "y":
          if (data) {
            event.preventDefault();
            copyLeafCitation(data.slug);
          }
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
          } else if (onNextReference) {
            event.preventDefault();
            onNextReference();
          }
          break;
        case "h":
        case "ArrowLeft":
          if (showRefs) {
            event.preventDefault();
            if (refFocus === REVIEW_REF_FOCUS.CONTENT) setRefFocus(REVIEW_REF_FOCUS.LIST);
            else setShowRefs(false);
          } else if (onPreviousReference) {
            event.preventDefault();
            onPreviousReference();
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
        case "g":
          event.preventDefault();
          if (showRefs && refFocus === REVIEW_REF_FOCUS.LIST) {
            setRefSel(0);
          } else {
            scrollEdge(false);
          }
          break;
        case "G":
          event.preventDefault();
          if (showRefs && refFocus === REVIEW_REF_FOCUS.LIST) {
            setRefSel(count ? count - 1 : 0);
          } else {
            scrollEdge(true);
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
        case "f":
        case "F":
          if (showRefs && count > 0 && onOpenReferenceFullPage) {
            event.preventDefault();
            onOpenReferenceFullPage();
          }
          break;
        default:
          break;
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [
    data,
    onBack,
    onNextReference,
    onOpenReferenceFullPage,
    onPreviousReference,
    refFocus,
    refReadRef,
    setRefFocus,
    setRefSel,
    setShowRefs,
    showRefs,
  ]);
}
