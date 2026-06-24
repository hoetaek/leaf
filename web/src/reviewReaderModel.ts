import type { ReviewRefFocus, ReviewResponse } from "./types";

export const REVIEW_REF_FOCUS = Object.freeze({
  LIST: "list",
  CONTENT: "content",
} satisfies Record<string, ReviewRefFocus>);

function clamp(value: number, min: number, max: number): number {
  return Math.min(Math.max(value, min), max);
}

export function reviewResourcePath(slug: string | null | undefined): string | null {
  return slug ? `/api/review/${slug}` : null;
}

export function referenceCount(data: Pick<ReviewResponse, "references"> | null | undefined): number {
  return data?.references?.length || 0;
}

export function clampReferenceIndex(index: number, count: number): number {
  if (count <= 0) return 0;
  return clamp(index, 0, count - 1);
}

export function nextReferenceIndex(index: number, step: number, count: number): number {
  return clampReferenceIndex(index + step, count);
}

export function isTextEntryElement(element: Pick<Element, "tagName"> | null): boolean {
  const tagName = element?.tagName;
  return tagName === "INPUT" || tagName === "TEXTAREA";
}

export function readingProgressFromRect(rect: Pick<DOMRect, "top" | "height">, viewportHeight: number): number {
  const total = rect.height - viewportHeight;
  return clamp(-rect.top / (total || 1), 0, 1);
}

export function progressWidth(progress: number): string {
  return `${(progress * 100).toFixed(1)}%`;
}
