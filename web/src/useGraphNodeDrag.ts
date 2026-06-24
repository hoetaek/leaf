import { useCallback, useRef } from "react";
import type { MutableRefObject, PointerEvent as ReactPointerEvent } from "react";
import { clampGraphPoint, graphNodeRadius } from "./graphPhysics";
import type { GraphLayoutNode } from "./types";
import type { GraphSimulation } from "./useForceGraphLayout";

interface DragState {
  id: string;
  pointerId: number;
  x: number;
  y: number;
  moved: boolean;
}

type GraphPointerEvent = ReactPointerEvent<SVGGElement>;

interface GraphNodeDragOptions {
  width: number;
  height: number;
  boundsPadding: number;
  graphPoint: (event: GraphPointerEvent) => [number, number];
  nodeRef: MutableRefObject<Map<string, GraphLayoutNode>>;
  simulationRef: MutableRefObject<GraphSimulation | null>;
  onSelectNode: (node: GraphLayoutNode) => void;
}

export function useGraphNodeDrag({
  width,
  height,
  boundsPadding,
  graphPoint,
  nodeRef,
  simulationRef,
  onSelectNode,
}: GraphNodeDragOptions) {
  const dragRef = useRef<DragState | null>(null);

  const beginDrag = useCallback(
    (event: GraphPointerEvent, node: GraphLayoutNode) => {
      if (event.button !== 0) return;
      event.stopPropagation();
      const active = nodeRef.current.get(node.id);
      if (!active) return;

      const [x, y] = graphPoint(event);
      active.fx = active.x;
      active.fy = active.y;
      dragRef.current = { id: node.id, pointerId: event.pointerId, x, y, moved: false };
      event.currentTarget.setPointerCapture(event.pointerId);
      simulationRef.current?.alphaTarget(0.28).restart();
    },
    [graphPoint, nodeRef, simulationRef],
  );

  const moveDrag = useCallback(
    (event: GraphPointerEvent) => {
      const drag = dragRef.current;
      if (!drag || drag.pointerId !== event.pointerId) return;

      event.preventDefault();
      const active = nodeRef.current.get(drag.id);
      if (!active) return;

      const [x, y] = graphPoint(event);
      const bounded = clampGraphPoint(x, y, {
        width,
        height,
        padding: boundsPadding,
        radius: graphNodeRadius(active),
      });
      if (Math.hypot(bounded.x - drag.x, bounded.y - drag.y) > 3) drag.moved = true;
      active.fx = bounded.x;
      active.fy = bounded.y;
    },
    [boundsPadding, graphPoint, height, nodeRef, width],
  );

  const endDrag = useCallback(
    (event: GraphPointerEvent, node: GraphLayoutNode) => {
      const drag = dragRef.current;
      if (!drag || drag.pointerId !== event.pointerId) return;

      event.stopPropagation();
      dragRef.current = null;
      event.currentTarget.releasePointerCapture(event.pointerId);

      const active = nodeRef.current.get(node.id);
      if (active) {
        active.fx = null;
        active.fy = null;
      }
      simulationRef.current?.alphaTarget(0);

      if (!drag.moved) {
        onSelectNode(node);
      }
    },
    [nodeRef, onSelectNode, simulationRef],
  );

  return { beginDrag, moveDrag, endDrag };
}
