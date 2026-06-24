import { useEffect, useState } from "react";
import { fetchJson } from "./api";

export function useJsonResource<T>(path: string | null) {
  const [data, setData] = useState<T | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let alive = true;
    setData(null);
    setError(null);
    if (!path) return undefined;

    fetchJson<T>(path)
      .then((nextData) => {
        if (!alive) return;
        setData(nextData);
        setError(null);
      })
      .catch((nextError) => {
        if (alive) setError(nextError instanceof Error ? nextError.message : String(nextError));
      });

    return () => {
      alive = false;
    };
  }, [path]);

  return { data, error };
}
