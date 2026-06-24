export const GRAPH_VIEWPORT = Object.freeze({
  width: 880,
  height: 520,
  boundsPadding: 36,
  minZoom: 0.45,
  maxZoom: 3.4,
});

function endpointDegree(endpoint) {
  return typeof endpoint === "string" ? 0 : endpoint?.degree || 0;
}

export function graphChargeStrength(node) {
  return -118 - (node?.degree || 0) * 24;
}

export function graphLinkDistance(link) {
  const degree = Math.min(endpointDegree(link.source) + endpointDegree(link.target), 6);
  return 88 + Math.max(0, 6 - degree) * 8;
}

export function createInitialGraphLayout(model, { width = GRAPH_VIEWPORT.width, height = GRAPH_VIEWPORT.height } = {}) {
  const sourceNodes = model?.nodes || [];
  const sourceLinks = model?.links || [];
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
