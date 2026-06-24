import { useCallback, useEffect, useRef, useState } from "react";
import type { ReviewResponse } from "./types";

export function useActiveReviewSection(data: ReviewResponse | null) {
  const [active, setActive] = useState(0);
  const sectionRefs = useRef<Array<HTMLElement | null>>([]);

  useEffect(() => {
    if (!data) return undefined;

    const observer = new IntersectionObserver(
      (entries) => {
        entries.forEach((entry) => {
          if (entry.isIntersecting) {
            const index = Number((entry.target as HTMLElement).dataset.idx);
            if (!Number.isNaN(index)) setActive(index);
          }
        });
      },
      { rootMargin: "-64px 0px -70% 0px", threshold: 0 },
    );
    sectionRefs.current.forEach((section) => section && observer.observe(section));
    return () => observer.disconnect();
  }, [data]);

  const jump = useCallback((index: number) => {
    sectionRefs.current[index]?.scrollIntoView({ behavior: "smooth", block: "start" });
  }, []);

  return { active, sectionRefs, jump };
}
