import type { ReviewRefFocus, ReviewResponse, ReviewSource } from "./types";

// The four LEAF phases shown as pipeline bars, in order. The `Status` source
// (00-status.md) is intentionally excluded — it has no gate sequence.
export const PIPELINE_PHASES = ["Learn", "Example", "Architect", "Feedback"] as const;

export interface PhaseProgress {
  phase: string;
  done: number;
  total: number;
  state: "done" | "partial" | "zero";
}

// Derive each phase's done/total from the canonical sources by counting
// `present` gates per phase. Reflects real partial completion (never a fake
// all-done), and skips the Status source so no spurious 5th bar appears.
export function computePhasePipeline(sources: ReviewSource[]): PhaseProgress[] {
  return PIPELINE_PHASES.map((phase) => {
    const gates = sources.filter((source) => source.phase === phase);
    const done = gates.filter((source) => source.present).length;
    const total = gates.length;
    const state = total > 0 && done === total ? "done" : done === 0 ? "zero" : "partial";
    return { phase, done, total, state };
  });
}

export type LeafStamp = "pressed" | "sprout" | "leaf" | "fallen";

// Fallen wins first: a pressed leaf that is later fallen (e.g. the supersede
// flow) keeps its pressed.md, so the API reports stage:"fallen" + pressed:true —
// the archived state must take precedence over the pressed stamp. Otherwise
// pressed.md wins, then the stamp follows the stage.
export function leafStamp(stage: string | undefined, pressed: boolean | undefined): LeafStamp {
  if (stage === "fallen") return "fallen";
  if (pressed) return "pressed";
  if (stage === "sprout") return "sprout";
  return "leaf";
}

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
