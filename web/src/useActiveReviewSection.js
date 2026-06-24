import { useCallback, useEffect, useRef, useState } from "react";

export function useActiveReviewSection(data) {
  const [active, setActive] = useState(0);
  const sectionRefs = useRef([]);

  useEffect(() => {
    if (!data) return undefined;

    const observer = new IntersectionObserver(
      (entries) => {
        entries.forEach((entry) => {
          if (entry.isIntersecting) {
            const index = Number(entry.target.dataset.idx);
            if (!Number.isNaN(index)) setActive(index);
          }
        });
      },
      { rootMargin: "-64px 0px -70% 0px", threshold: 0 },
    );
    sectionRefs.current.forEach((section) => section && observer.observe(section));
    return () => observer.disconnect();
  }, [data]);

  const jump = useCallback((index) => {
    sectionRefs.current[index]?.scrollIntoView({ behavior: "smooth", block: "start" });
  }, []);

  return { active, sectionRefs, jump };
}
