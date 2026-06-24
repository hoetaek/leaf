export const REVIEW_REF_FOCUS = Object.freeze({
  LIST: "list",
  CONTENT: "content",
});

function clamp(value, min, max) {
  return Math.min(Math.max(value, min), max);
}

export function reviewResourcePath(slug) {
  return slug ? `/api/review/${slug}` : null;
}

export function referenceCount(data) {
  return data?.references?.length || 0;
}

export function clampReferenceIndex(index, count) {
  if (count <= 0) return 0;
  return clamp(index, 0, count - 1);
}

export function nextReferenceIndex(index, step, count) {
  return clampReferenceIndex(index + step, count);
}

export function isTextEntryElement(element) {
  const tagName = element?.tagName;
  return tagName === "INPUT" || tagName === "TEXTAREA";
}

export function readingProgressFromRect(rect, viewportHeight) {
  const total = rect.height - viewportHeight;
  return clamp(-rect.top / (total || 1), 0, 1);
}

export function progressWidth(progress) {
  return `${(progress * 100).toFixed(1)}%`;
}
