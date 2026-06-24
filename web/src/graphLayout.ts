import type { GraphLayout, GraphModel } from "./types";

export const GRAPH_VIEWPORT: {
  width: number;
  height: number;
  boundsPadding: number;
  minZoom: number;
  maxZoom: number;
} = Object.freeze({
  width: 880,
  height: 520,
  boundsPadding: 36,
  minZoom: 0.45,
  maxZoom: 3.4,
});

type DegreeEndpoint = string | { degree?: number } | null | undefined;

function endpointDegree(endpoint: DegreeEndpoint): number {
  return typeof endpoint === "string" ? 0 : endpoint?.degree || 0;
}

export function graphChargeStrength(node: { degree?: number } | null | undefined): number {
  return -118 - (node?.degree || 0) * 24;
}

export function graphLinkDistance(link: { source: DegreeEndpoint; target: DegreeEndpoint }): number {
  const degree = Math.min(endpointDegree(link.source) + endpointDegree(link.target), 6);
  return 88 + Math.max(0, 6 - degree) * 8;
}

export function createInitialGraphLayout(
  model: GraphModel,
  { width = GRAPH_VIEWPORT.width, height = GRAPH_VIEWPORT.height }: { width?: number; height?: number } = {},
): GraphLayout {
  const sourceNodes = model.nodes;
  const sourceLinks = model.links;
  const ring = Math.min(width, height) * 0.28;
  const count = Math.max(sourceNodes.length, 1);

  return {
    nodes: sourceNodes.map((node, index) => {
      const angle = (index / count) * Math.PI * 2 - Math.PI / 2;
      return {
        ...node,
        x: width / 2 + Math.cos(angle) * ring,
        y: height / 2 + Math.sin(angle) * ring,
      };
    }),
    links: sourceLinks.map((link) => ({ ...link })),
  };
}
