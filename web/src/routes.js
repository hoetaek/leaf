export function leafHref(slug) {
  return `#/leaf/${encodeURIComponent(slug)}`;
}

export function openLeaf(slug) {
  window.location.hash = leafHref(slug);
}
