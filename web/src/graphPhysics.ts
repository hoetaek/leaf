import type { Force } from "d3-force";
import type { GraphLayoutNode, GraphLink, Point } from "./types";

interface GraphBoundsOptions {
  width: number;
  height: number;
  padding?: number;
  radius?: number;
}

interface ForceGraphBoundsOptions {
  padding?: number;
  radius?: (node: GraphLayoutNode) => number;
}

function clamp(value: number, min: number, max: number): number {
  return Math.min(Math.max(value, min), max);
}

export function graphNodeRadius(node: { degree?: number } | null | undefined): number {
  return 6 + Math.min(11, Math.sqrt(node?.degree || 0) * 3.2);
}

export function clampGraphPoint(
  x: number,
  y: number,
  { width, height, padding = 0, radius = 0 }: GraphBoundsOptions,
): Point {
  const minX = padding + radius;
  const minY = padding + radius;
  const maxX = Math.max(minX, width - padding - radius);
  const maxY = Math.max(minY, height - padding - radius);

  return {
    x: clamp(x, minX, maxX),
    y: clamp(y, minY, maxY),
  };
}

export function constrainGraphNode<T extends Point & { vx?: number; vy?: number }>(
  node: T,
  options: GraphBoundsOptions,
): T {
  const next = clampGraphPoint(node.x, node.y, options);

  if (next.x !== node.x) {
    node.x = next.x;
    node.vx = 0;
  }
  if (next.y !== node.y) {
    node.y = next.y;
    node.vy = 0;
  }

  return node;
}

export function forceGraphBounds(
  width: number,
  height: number,
  { padding = 40, radius = graphNodeRadius }: ForceGraphBoundsOptions = {},
): Force<GraphLayoutNode, GraphLink> {
  let nodes: GraphLayoutNode[] = [];
  const force = () => {
    for (const node of nodes) {
      constrainGraphNode(node, {
        width,
        height,
        padding,
        radius: radius(node),
      });
    }
  };
  force.initialize = (nextNodes: GraphLayoutNode[]) => {
    nodes = nextNodes || [];
  };
  return force;
}
