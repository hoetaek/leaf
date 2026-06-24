import { fetchJson } from "./api";
import type { WorkspacePreviewResponse } from "./types";

const workspacePreviewCache = new Map<string, WorkspacePreviewResponse>();
const workspacePreviewInflight = new Map<string, Promise<WorkspacePreviewResponse>>();

function previewPath(slug: string) {
  return `/api/preview/${encodeURIComponent(slug)}`;
}

export function cachedWorkspacePreview(slug: string): WorkspacePreviewResponse | undefined {
  return workspacePreviewCache.get(slug);
}

export function loadWorkspacePreview(slug: string): Promise<WorkspacePreviewResponse> {
  const cached = workspacePreviewCache.get(slug);
  if (cached) return Promise.resolve(cached);

  const inflight = workspacePreviewInflight.get(slug);
  if (inflight) return inflight;

  const request = fetchJson<WorkspacePreviewResponse>(previewPath(slug))
    .then((data) => {
      workspacePreviewCache.set(slug, data);
      return data;
    })
    .finally(() => {
      workspacePreviewInflight.delete(slug);
    });

  workspacePreviewInflight.set(slug, request);
  return request;
}

export function clearWorkspacePreviewCacheForTest() {
  workspacePreviewCache.clear();
  workspacePreviewInflight.clear();
}
