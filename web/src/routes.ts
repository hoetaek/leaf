export function leafHref(slug: string): string {
  return `#/leaf/${encodeURIComponent(slug)}`;
}

export function openLeaf(slug: string): void {
  window.location.hash = leafHref(slug);
}
