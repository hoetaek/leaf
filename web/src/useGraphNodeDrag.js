import { useCallback, useRef } from "react";
import { clampGraphPoint, graphNodeRadius } from "./graphPhysics.js";

export function useGraphNodeDrag({ width, height, boundsPadding, graphPoint, nodeRef, simulationRef, onSelectNode }) {
  const dragRef = useRef(null);

  const beginDrag = useCallback(
    (event, node) => {
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
    (event) => {
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
    (event, node) => {
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
