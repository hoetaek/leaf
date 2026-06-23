export function buildGraphModel(data = {}) {
  const sourceNodes = Array.isArray(data.nodes) ? data.nodes : [];
  const sourceEdges = Array.isArray(data.edges) ? data.edges : [];
  const nodes = sourceNodes.map((node) => ({
    ...node,
    id: node.id,
    slug: node.slug || node.id,
    title: node.title || node.slug || node.id,
    description: node.description || "",
    tags: Array.isArray(node.tags) ? node.tags : [],
    degree: 0,
  }));

  const nodeById = new Map(nodes.map((node) => [node.id, node]));
  const neighboursById = new Map(nodes.map((node) => [node.id, new Set([node.id])]));
  const links = [];

  for (const edge of sourceEdges) {
    if (!nodeById.has(edge.source) || !nodeById.has(edge.target)) {
      continue;
    }

    links.push({
      source: edge.source,
      target: edge.target,
      predicate: edge.predicate || "related_to",
      note: edge.note || "",
      path: edge.path || "",
    });

    const source = nodeById.get(edge.source);
    const target = nodeById.get(edge.target);
    source.degree += 1;
    target.degree += 1;
    neighboursById.get(edge.source).add(edge.target);
    neighboursById.get(edge.target).add(edge.source);
  }

  return {
    nodes,
    links,
    nodeById,
    neighboursById,
  };
}
