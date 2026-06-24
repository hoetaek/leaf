import { useEffect, useRef, useState } from "react";
import { readingProgressFromRect } from "./reviewReaderModel.js";

export function useReadingProgress(data, reportRef) {
  const [progress, setProgress] = useState(0);
  const frameRef = useRef(null);

  useEffect(() => {
    const onScroll = () => {
      if (frameRef.current) return;
      frameRef.current = requestAnimationFrame(() => {
        frameRef.current = null;
        const report = reportRef.current;
        if (!report) return;
        setProgress(readingProgressFromRect(report.getBoundingClientRect(), window.innerHeight));
      });
    };
    window.addEventListener("scroll", onScroll, { passive: true });
    onScroll();
    return () => {
      window.removeEventListener("scroll", onScroll);
      if (frameRef.current) cancelAnimationFrame(frameRef.current);
    };
  }, [data, reportRef]);

  return progress;
}
