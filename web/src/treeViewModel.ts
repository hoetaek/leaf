import type { GraphApiResponse, LeafStage, WorkspaceListResponse, WorkspaceRow } from "./types";
import { buildWorkspaceRows, workspaceCounts } from "./workspaceModel";

export interface TreeStageDefinition {
  key: LeafStage;
  label: string;
  stateLabel: string;
  emptyLabel: string;
}

export interface TreeStageSummary extends TreeStageDefinition {
  count: number;
  items: WorkspaceRow[];
}

export interface TreeViewModel {
  stages: TreeStageSummary[];
  counts: Record<string, number>;
  totalWork: number;
  pressedCount: number | null;
  edgeCount: number | null;
  maturityLabel: string;
  activeCount: number;
  completionCount: number;
}

export const TREE_STAGES: TreeStageDefinition[] = [
  { key: "sprouts", label: "Sprouts", stateLabel: "active", emptyLabel: "No active sprouts" },
  { key: "leaves", label: "Leaves", stateLabel: "landed", emptyLabel: "No landed leaves" },
  { key: "fallen", label: "Fallen", stateLabel: "archived", emptyLabel: "No fallen work" },
];

export function treeMaturityLabel(leafCount: number): string {
  if (leafCount <= 0) return "seedling";
  if (leafCount <= 3) return "young";
  if (leafCount <= 10) return "branching";
  if (leafCount <= 30) return "grown";
  return "mature";
}

export function treeItemPhaseLabel(row: WorkspaceRow): string {
  const phase = row.status?.current_phase;
  const gate = row.status?.current_gate;
  if (phase && gate) return `${phase} · ${gate}`;
  if (phase) return phase;
  if (gate) return gate;
  return row.status?.next_action || "—";
}

export function buildTreeViewModel(
  workspace: WorkspaceListResponse | null | undefined,
  graph: GraphApiResponse | null | undefined,
): TreeViewModel {
  const rows = buildWorkspaceRows(workspace);
  const counts = workspaceCounts(workspace);
  const stageItems = new Map<LeafStage, WorkspaceRow[]>(TREE_STAGES.map((stage) => [stage.key, []]));

  for (const row of rows) {
    if (row._stage === "sprouts" || row._stage === "leaves" || row._stage === "fallen") {
      stageItems.get(row._stage)?.push(row);
    }
  }

  const stages = TREE_STAGES.map((stage) => ({
    ...stage,
    count: counts[stage.key] || 0,
    items: stageItems.get(stage.key) || [],
  }));
  const leaves = counts.leaves || 0;

  return {
    stages,
    counts,
    totalWork: (counts.sprouts || 0) + leaves + (counts.fallen || 0),
    pressedCount: Array.isArray(graph?.nodes) ? graph.nodes.length : null,
    edgeCount: Array.isArray(graph?.edges) ? graph.edges.length : null,
    maturityLabel: treeMaturityLabel(leaves),
    activeCount: counts.sprouts || 0,
    completionCount: leaves,
  };
}
