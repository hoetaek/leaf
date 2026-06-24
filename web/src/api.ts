export function fetchJson<T>(path: string): Promise<T> {
  return fetch(path).then((response) => {
    if (!response.ok) throw new Error(`HTTP ${response.status}`);
    return response.json() as Promise<T>;
  });
}
