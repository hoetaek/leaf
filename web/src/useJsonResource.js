import { useEffect, useState } from "react";
import { fetchJson } from "./api.js";

export function useJsonResource(path) {
  const [data, setData] = useState(null);
  const [error, setError] = useState(null);

  useEffect(() => {
    if (!path) return undefined;

    let alive = true;
    fetchJson(path)
      .then((nextData) => {
        if (!alive) return;
        setData(nextData);
        setError(null);
      })
      .catch((nextError) => {
        if (alive) setError(nextError.message);
      });

    return () => {
      alive = false;
    };
  }, [path]);

  return { data, error };
}
