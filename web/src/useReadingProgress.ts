import { useEffect, useRef, useState } from "react";
import type { RefObject } from "react";
import { readingProgressFromRect } from "./reviewReaderModel";

export function useReadingProgress(data: unknown, reportRef: RefObject<HTMLElement>) {
  const [progress, setProgress] = useState(0);
  const frameRef = useRef<number | null>(null);

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
