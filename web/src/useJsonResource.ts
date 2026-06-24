import { useEffect, useState } from "react";
import { fetchJson } from "./api";

export function useJsonResource<T>(path: string | null) {
  const [resource, setResource] = useState<{
    data: T | null;
    error: string | null;
    path: string | null;
  }>({ data: null, error: null, path: null });

  useEffect(() => {
    let alive = true;
    if (!path) return undefined;

    fetchJson<T>(path)
      .then((nextData) => {
        if (!alive) return;
        setResource({ data: nextData, error: null, path });
      })
      .catch((nextError) => {
        if (alive) {
          setResource({
            data: null,
            error: nextError instanceof Error ? nextError.message : String(nextError),
            path,
          });
        }
      });

    return () => {
      alive = false;
    };
  }, [path]);

  return {
    data: resource.path === path ? resource.data : null,
    error: resource.path === path ? resource.error : null,
  };
}
