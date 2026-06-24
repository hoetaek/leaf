export function leafHref(slug: string): string {
  return `#/leaf/${encodeURIComponent(slug)}`;
}

export function openLeaf(slug: string): void {
  window.location.hash = leafHref(slug);
}

export function referenceHref(slug: string, relativePath: string): string {
  return `${leafHref(slug)}/ref/${encodeURIComponent(relativePath)}`;
}

export function openReference(slug: string, relativePath: string): void {
  window.location.hash = referenceHref(slug, relativePath);
}
