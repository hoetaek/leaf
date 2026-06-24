const VISITED_KEY = "leaf-graph-visited";

function browserStorage() {
  if (typeof window === "undefined") return null;
  return window.localStorage;
}

export function readVisitedLeaves(storage = browserStorage()) {
  if (!storage) return new Set();

  try {
    const raw = storage.getItem(VISITED_KEY);
    const parsed = raw ? JSON.parse(raw) : [];
    return new Set(Array.isArray(parsed) ? parsed : []);
  } catch {
    return new Set();
  }
}

export function writeVisitedLeaves(ids, storage = browserStorage()) {
  if (!storage) return;

  try {
    storage.setItem(VISITED_KEY, JSON.stringify([...ids]));
  } catch {
    // localStorage can be unavailable in private or restricted contexts.
  }
}
