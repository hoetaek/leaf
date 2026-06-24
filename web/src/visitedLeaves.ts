const VISITED_KEY = "leaf-graph-visited";

type VisitedStorage = Pick<Storage, "getItem" | "setItem">;

function browserStorage(): VisitedStorage | null {
  if (typeof window === "undefined") return null;
  return window.localStorage;
}

export function readVisitedLeaves(storage: VisitedStorage | null = browserStorage()): Set<string> {
  if (!storage) return new Set();

  try {
    const raw = storage.getItem(VISITED_KEY);
    const parsed = raw ? JSON.parse(raw) : [];
    return new Set(Array.isArray(parsed) ? parsed.filter((id): id is string => typeof id === "string") : []);
  } catch {
    return new Set();
  }
}

export function writeVisitedLeaves(ids: Iterable<string>, storage: VisitedStorage | null = browserStorage()): void {
  if (!storage) return;

  try {
    storage.setItem(VISITED_KEY, JSON.stringify([...ids]));
  } catch {
    // localStorage can be unavailable in private or restricted contexts.
  }
}
