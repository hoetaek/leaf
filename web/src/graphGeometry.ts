import type { Point } from "./types";

interface DirectedEdgeGeometryOptions {
  sourceRadius?: number;
  targetRadius?: number;
  labelT?: number;
  labelOffset?: number;
}

interface DirectedEdgeGeometry {
  start: Point;
  end: Point;
  label: Point;
  path: string;
}

function round(value: number): number {
  return Number(value.toFixed(3));
}

export function buildDirectedEdgeGeometry(
  source: Point,
  target: Point,
  options: DirectedEdgeGeometryOptions = {},
): DirectedEdgeGeometry {
  const sourceRadius = options.sourceRadius || 0;
  const targetRadius = options.targetRadius || 0;
  const labelT = options.labelT ?? 0.64;
  const labelOffset = options.labelOffset ?? 10;
  const dx = target.x - source.x;
  const dy = target.y - source.y;
  const length = Math.hypot(dx, dy);

  if (!length) {
    const point = { x: source.x, y: source.y };
    return {
      start: point,
      end: point,
      label: { x: source.x, y: source.y },
      path: `M ${source.x} ${source.y} L ${source.x} ${source.y}`,
    };
  }

  const ux = dx / length;
  const uy = dy / length;
  const nx = -uy;
  const ny = ux;
  const start = {
    x: round(source.x + ux * sourceRadius),
    y: round(source.y + uy * sourceRadius),
  };
  const end = {
    x: round(target.x - ux * targetRadius),
    y: round(target.y - uy * targetRadius),
  };
  const label = {
    x: round(start.x + (end.x - start.x) * labelT + nx * labelOffset),
    y: round(start.y + (end.y - start.y) * labelT - Math.abs(ny || 1) * labelOffset),
  };

  return {
    start,
    end,
    label,
    path: `M ${start.x} ${start.y} L ${end.x} ${end.y}`,
  };
}
