export function fetchJson(path) {
  return fetch(path).then((response) => {
    if (!response.ok) throw new Error(`HTTP ${response.status}`);
    return response.json();
  });
}
