import { showToast } from "./Toast";

// The canonical way to cite a leaf: `leaf:<slug>` (matches the "인용: leaf:…"
// convention used across .leaf status/pressed files).
export function leafCitation(slug: string): string {
  return `leaf:${slug}`;
}

// Copy a leaf's citation token to the clipboard and confirm with a toast.
// Read-only: this is a pure client-side action and never touches `.leaf`.
// `leaf serve` runs on localhost (a secure context) so the Clipboard API is
// available; if it is missing or denied, fail quietly without breaking the app.
export function copyLeafCitation(slug: string): void {
  const citation = leafCitation(slug);
  const result = navigator.clipboard?.writeText(citation);
  if (!result) return;
  void result.then(
    () => showToast(`copied ${citation}`),
    () => {},
  );
}
