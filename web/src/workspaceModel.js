export const WORKSPACE_GATES = Array.from({ length: 10 }, (_, index) => index + 1);
export const WORKSPACE_STAGES = ["all", "sprouts", "leaves", "fallen"];

export function workspaceCounts(data) {
  return Object.fromEntries(Object.entries(data?.stages || {}).map(([key, value]) => [key, value.count]));
}

export function buildWorkspaceRows(data) {
  const rows = [];

  for (const [stage, stageData] of Object.entries(data?.stages || {})) {
    for (const item of stageData.items || []) {
      rows.push({ ...item, _stage: stage });
    }
  }

  return rows;
}

export function filterWorkspaceRows(data, { stage = "all", query = "" } = {}) {
  const normalizedQuery = query.trim().toLowerCase();

  return buildWorkspaceRows(data).filter((item) => {
    const matchesStage = stage === "all" || item._stage === stage;
    const haystack = `${item.slug} ${item.status?.next_action || ""}`.toLowerCase();
    return matchesStage && (!normalizedQuery || haystack.includes(normalizedQuery));
  });
}

export function clampWorkspaceSelection(index, rowCount) {
  if (rowCount <= 0) return 0;
  return Math.min(Math.max(0, index), rowCount - 1);
}
