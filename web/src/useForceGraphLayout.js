import { useEffect, useRef, useState } from "react";
import { forceCenter, forceCollide, forceLink, forceManyBody, forceSimulation } from "d3-force";
import { createInitialGraphLayout, graphChargeStrength, graphLinkDistance } from "./graphLayout.js";
import { constrainGraphNode, forceGraphBounds, graphNodeRadius } from "./graphPhysics.js";

const EMPTY_LAYOUT = { nodes: [], links: [] };

export function useForceGraphLayout(model, { width, height, boundsPadding }) {
  const [layout, setLayout] = useState(EMPTY_LAYOUT);
  const nodeRef = useRef(new Map());
  const simulationRef = useRef(null);

  useEffect(() => {
    simulationRef.current?.stop();

    if (!model) {
      nodeRef.current = new Map();
      simulationRef.current = null;
      return undefined;
    }

    const { nodes, links } = createInitialGraphLayout(model, { width, height });
    nodeRef.current = new Map(nodes.map((node) => [node.id, node]));

    const constrainNodes = () => {
      for (const node of nodes) {
        constrainGraphNode(node, {
          width,
          height,
          padding: boundsPadding,
          radius: graphNodeRadius(node),
        });
      }
    };

    let frame = null;
    const publish = () => {
      frame = null;
      setLayout({ nodes: [...nodes], links: [...links] });
    };
    const schedule = () => {
      constrainNodes();
      if (frame === null) {
        frame = window.requestAnimationFrame(publish);
      }
    };
    schedule();

    if (!nodes.length) {
      return () => {
        if (frame !== null) window.cancelAnimationFrame(frame);
      };
    }

    const simulation = forceSimulation(nodes)
      .force("charge", forceManyBody().strength(graphChargeStrength))
      .force(
        "link",
        forceLink(links)
          .id((node) => node.id)
          .distance(graphLinkDistance)
          .strength(0.46),
      )
      .force("center", forceCenter(width / 2, height / 2).strength(0.16))
      .force("collide", forceCollide((node) => graphNodeRadius(node) + 18).iterations(2))
      .force("bounds", forceGraphBounds(width, height, { padding: boundsPadding, radius: graphNodeRadius }))
      .alpha(1)
      .alphaDecay(0.035)
      .on("tick", schedule);

    simulationRef.current = simulation;

    return () => {
      if (frame !== null) window.cancelAnimationFrame(frame);
      simulation.stop();
      if (simulationRef.current === simulation) simulationRef.current = null;
    };
  }, [boundsPadding, height, model, width]);

  return { layout, nodeRef, simulationRef };
}
