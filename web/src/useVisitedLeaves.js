import { useCallback, useState } from "react";
import { readVisitedLeaves, writeVisitedLeaves } from "./visitedLeaves.js";

export function useVisitedLeaves() {
  const [visited, setVisited] = useState(() => readVisitedLeaves());

  const markVisited = useCallback((id) => {
    if (!id) return;

    setVisited((current) => {
      if (current.has(id)) return current;
      const next = new Set(current);
      next.add(id);
      writeVisitedLeaves(next);
      return next;
    });
  }, []);

  return { visited, markVisited };
}
