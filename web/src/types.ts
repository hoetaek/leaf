import type { SimulationLinkDatum, SimulationNodeDatum } from "d3-force";

export interface GraphApiNode {
  id: string;
  slug?: string;
  title?: string;
  description?: string;
  tags?: unknown;
  [key: string]: unknown;
}

export interface GraphApiEdge {
  source: string;
  target: string;
  predicate?: string;
  note?: string;
  path?: string;
  [key: string]: unknown;
}

export interface GraphApiResponse {
  nodes?: GraphApiNode[];
  edges?: GraphApiEdge[];
}

export interface GraphNode extends SimulationNodeDatum {
  id: string;
  slug: string;
  title: string;
  description: string;
  tags: string[];
  degree: number;
}

export type GraphLayoutNode = GraphNode & {
  x: number;
  y: number;
};

export interface GraphLink extends SimulationLinkDatum<GraphLayoutNode> {
  source: string | GraphLayoutNode;
  target: string | GraphLayoutNode;
  predicate: string;
  note: string;
  path: string;
}

export interface GraphModel {
  nodes: GraphNode[];
  links: GraphLink[];
  nodeById: Map<string, GraphNode>;
  neighboursById: Map<string, Set<string>>;
}

export interface GraphLayout {
  nodes: GraphLayoutNode[];
  links: GraphLink[];
}

export interface Point {
  x: number;
  y: number;
}

export interface ZoomTransformState extends Point {
  k: number;
}

export interface ZoomResult extends ZoomTransformState {
  changed: boolean;
}

export type LeafStage = "sprouts" | "leaves" | "fallen";
export type WorkspaceStageFilter = "all" | LeafStage;

export interface LeafStatus {
  current_phase?: string;
  current_gate?: string;
  next_action?: string;
  parse_state?: string;
  progress_done?: number;
  progress_current?: number;
  progress_label?: string;
  [key: string]: unknown;
}

export interface WorkspaceItem {
  slug: string;
  path?: string;
  status?: LeafStatus;
  [key: string]: unknown;
}

export type WorkspaceRow = WorkspaceItem & {
  _stage: string;
};

export interface WorkspaceStageData {
  count: number;
  items?: WorkspaceItem[];
}

export interface WorkspaceListResponse {
  workspace_name?: string;
  stages?: Record<string, WorkspaceStageData | undefined>;
}

export type WorkspacePreviewLine =
  | { kind: "heading"; level: number; text: string }
  | { kind: "source_boundary"; phase: string; gate: string; source: string }
  | { kind: "checkbox"; marker: string; checked: boolean; text: string }
  | { kind: "list_item"; marker: string; text: string }
  | { kind: "code"; text: string }
  | { kind: "table"; text: string }
  | { kind: "text"; text: string };

export interface WorkspacePreviewResponse {
  title: string;
  lines: WorkspacePreviewLine[];
}

export interface ReviewSource {
  gate: string;
  phase: string;
  relative_path: string;
  present: boolean;
  markdown: string;
}

export interface ReviewReference {
  relative_path: string;
  markdown: string;
}

export interface ReviewResponse {
  slug: string;
  sources: ReviewSource[];
  references?: ReviewReference[];
  // Status preamble surfaced as typed fields by the Rust server (single source
  // of truth). why/what/wireframe are omitted when absent or `none — …`.
  stage?: string;
  pressed?: boolean;
  why?: string;
  what?: string;
  wireframe?: string;
}

export type ReviewRefFocus = "list" | "content";
