import type { WorkspaceListResponse, WorkspaceRow, WorkspaceStageFilter } from "./types";

export const WORKSPACE_GATES = Array.from({ length: 10 }, (_, index) => index + 1);
export const WORKSPACE_STAGES: WorkspaceStageFilter[] = ["all", "sprouts", "leaves", "fallen"];

export function workspaceCounts(data: WorkspaceListResponse | null | undefined): Record<string, number> {
  return Object.fromEntries(Object.entries(data?.stages || {}).map(([key, value]) => [key, value?.count ?? 0]));
}

export function buildWorkspaceRows(data: WorkspaceListResponse | null | undefined): WorkspaceRow[] {
  const rows: WorkspaceRow[] = [];

  for (const [stage, stageData] of Object.entries(data?.stages || {})) {
    for (const item of stageData?.items || []) {
      rows.push({ ...item, _stage: stage });
    }
  }

  return rows;
}

export function filterWorkspaceRows(
  data: WorkspaceListResponse | null | undefined,
  { stage = "all", query = "" }: { stage?: WorkspaceStageFilter; query?: string } = {},
): WorkspaceRow[] {
  const normalizedQuery = query.trim().toLowerCase();

  return buildWorkspaceRows(data).filter((item) => {
    const matchesStage = stage === "all" || item._stage === stage;
    const haystack = `${item.slug} ${item.status?.next_action || ""}`.toLowerCase();
    return matchesStage && (!normalizedQuery || haystack.includes(normalizedQuery));
  });
}

export function clampWorkspaceSelection(index: number, rowCount: number): number {
  if (rowCount <= 0) return 0;
  return Math.min(Math.max(0, index), rowCount - 1);
}
