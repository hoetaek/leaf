import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { forceCenter, forceCollide, forceLink, forceManyBody, forceSimulation } from "d3-force";
import { select } from "d3-selection";
import { zoom, zoomIdentity } from "d3-zoom";
import { fetchJson } from "./api.js";
import GraphCanvas from "./GraphCanvas.jsx";
import GraphDetailsPanel from "./GraphDetailsPanel.jsx";
import { buildGraphModel } from "./graphModel.js";
import { clampGraphPoint, constrainGraphNode, forceGraphBounds, graphNodeRadius } from "./graphPhysics.js";
import { nextWheelZoom } from "./graphZoom.js";

const W = 880;
const H = 520;
const GRAPH_BOUNDS_PADDING = 36;
const MIN_ZOOM = 0.45;
const MAX_ZOOM = 3.4;
const VISITED_KEY = "leaf-graph-visited";

function readVisited() {
  if (typeof window === "undefined") return new Set();
  try {
    const raw = window.localStorage.getItem(VISITED_KEY);
    const parsed = raw ? JSON.parse(raw) : [];
    return new Set(Array.isArray(parsed) ? parsed : []);
  } catch {
    return new Set();
  }
}

function writeVisited(ids) {
  try {
    window.localStorage.setItem(VISITED_KEY, JSON.stringify([...ids]));
  } catch {
    // localStorage can be unavailable in private or restricted contexts.
  }
}

function endpointDegree(endpoint) {
  return typeof endpoint === "string" ? 0 : endpoint?.degree || 0;
}

export default function GraphView() {
  const [data, setData] = useState(null);
  const [error, setError] = useState(null);
  const [selId, setSelId] = useState(null);
  const [hoverId, setHoverId] = useState(null);
  const [layout, setLayout] = useState({ nodes: [], links: [] });
  const [transform, setTransform] = useState(zoomIdentity);
  const [visited, setVisited] = useState(readVisited);
  const svgRef = useRef(null);
  const simulationRef = useRef(null);
  const zoomRef = useRef(null);
  const transformRef = useRef(zoomIdentity);
  const nodeRef = useRef(new Map());
  const dragRef = useRef(null);

  useEffect(() => {
    fetchJson("/api/graph")
      .then((d) => setData(d))
      .catch((e) => setError(e.message));
  }, []);

  const model = useMemo(() => (data ? buildGraphModel(data) : null), [data]);

  const effectiveSelId = selId && model?.nodeById.has(selId) ? selId : model?.nodes[0]?.id || null;
  const selected = effectiveSelId && model ? model.nodeById.get(effectiveSelId) : null;
  const focusIds = useMemo(() => {
    if (!hoverId || !model) return null;
    return model.neighboursById.get(hoverId) || null;
  }, [hoverId, model]);
  const visibleEdgeCount = model?.links.length || 0;
  const hiddenEdgeCount = data && model ? Math.max(0, data.edges.length - model.links.length) : 0;

  const selectNode = useCallback((node) => {
    if (!node?.id) return;
    setSelId(node.id);
    setVisited((current) => {
      if (current.has(node.id)) return current;
      const next = new Set(current);
      next.add(node.id);
      writeVisited(next);
      return next;
    });
  }, []);

  useEffect(() => {
    if (!model || !svgRef.current) return;

    const svg = select(svgRef.current);
    const zoomer = zoom()
      .scaleExtent([MIN_ZOOM, MAX_ZOOM])
      .on("zoom", (event) => {
        transformRef.current = event.transform;
        setTransform(event.transform);
      });

    zoomRef.current = zoomer;
    svg.call(zoomer);
    svg.on("wheel.zoom", null);
    svg.on("dblclick.zoom", null);

    return () => {
      svg.on(".zoom", null);
      zoomRef.current = null;
    };
  }, [model]);

  useEffect(() => {
    if (!model) return;

    simulationRef.current?.stop();

    const ring = Math.min(W, H) * 0.28;
    const count = Math.max(model.nodes.length, 1);
    const nodes = model.nodes.map((node, index) => {
      const angle = (index / count) * Math.PI * 2 - Math.PI / 2;
      return {
        ...node,
        x: W / 2 + Math.cos(angle) * ring,
        y: H / 2 + Math.sin(angle) * ring,
      };
    });
    const links = model.links.map((link) => ({ ...link }));

    nodeRef.current = new Map(nodes.map((node) => [node.id, node]));

    const constrainNodes = () => {
      for (const node of nodes) {
        constrainGraphNode(node, {
          width: W,
          height: H,
          padding: GRAPH_BOUNDS_PADDING,
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
      .force(
        "charge",
        forceManyBody().strength((node) => -118 - (node.degree || 0) * 24),
      )
      .force(
        "link",
        forceLink(links)
          .id((node) => node.id)
          .distance(
            (link) => 88 + Math.max(0, 6 - Math.min(endpointDegree(link.source) + endpointDegree(link.target), 6)) * 8,
          )
          .strength(0.46),
      )
      .force("center", forceCenter(W / 2, H / 2).strength(0.16))
      .force("collide", forceCollide((node) => graphNodeRadius(node) + 18).iterations(2))
      .force("bounds", forceGraphBounds(W, H, { padding: GRAPH_BOUNDS_PADDING, radius: graphNodeRadius }))
      .alpha(1)
      .alphaDecay(0.035)
      .on("tick", schedule);

    simulationRef.current = simulation;

    return () => {
      if (frame !== null) window.cancelAnimationFrame(frame);
      simulation.stop();
      if (simulationRef.current === simulation) simulationRef.current = null;
    };
  }, [model]);

  const layoutById = useMemo(() => new Map(layout.nodes.map((node) => [node.id, node])), [layout.nodes]);

  const svgPoint = useCallback((event) => {
    const svg = svgRef.current;
    if (!svg) return [0, 0];

    const point = svg.createSVGPoint();
    point.x = event.clientX;
    point.y = event.clientY;
    const matrix = svg.getScreenCTM();
    if (!matrix) return [0, 0];
    const screenPoint = point.matrixTransform(matrix.inverse());
    return [screenPoint.x, screenPoint.y];
  }, []);

  const graphPoint = useCallback(
    (event) => {
      return transformRef.current.invert(svgPoint(event));
    },
    [svgPoint],
  );

  const applyZoomTransform = useCallback((next) => {
    const nextTransform = zoomIdentity.translate(next.x, next.y).scale(next.k);
    if (svgRef.current && zoomRef.current) {
      select(svgRef.current).call(zoomRef.current.transform, nextTransform);
    } else {
      transformRef.current = nextTransform;
      setTransform(nextTransform);
    }
  }, []);

  const handleWheel = useCallback(
    (event) => {
      event.preventDefault();
      event.stopPropagation();
      const [x, y] = svgPoint(event);
      const current = transformRef.current;
      const next = nextWheelZoom(
        { x: current.x, y: current.y, k: current.k },
        { x, y },
        {
          deltaY: event.deltaY,
          deltaMode: event.deltaMode,
          ctrlKey: event.ctrlKey,
          minScale: MIN_ZOOM,
          maxScale: MAX_ZOOM,
        },
      );

      if (next.changed) {
        applyZoomTransform(next);
      }
    },
    [applyZoomTransform, svgPoint],
  );

  useEffect(() => {
    if (!model || !svgRef.current) return;

    const svg = svgRef.current;
    svg.addEventListener("wheel", handleWheel, { passive: false });
    return () => {
      svg.removeEventListener("wheel", handleWheel);
    };
  }, [handleWheel, model]);

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
    [graphPoint],
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
        width: W,
        height: H,
        padding: GRAPH_BOUNDS_PADDING,
        radius: graphNodeRadius(active),
      });
      if (Math.hypot(bounded.x - drag.x, bounded.y - drag.y) > 3) drag.moved = true;
      active.fx = bounded.x;
      active.fy = bounded.y;
    },
    [graphPoint],
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
        selectNode(node);
      }
    },
    [selectNode],
  );

  const resetView = useCallback(() => {
    if (!svgRef.current || !zoomRef.current) return;
    select(svgRef.current).call(zoomRef.current.transform, zoomIdentity);
  }, []);

  if (error) return <p className="err">그래프를 불러오지 못했습니다: {error}</p>;
  if (!data) return <p className="muted">불러오는 중…</p>;

  return (
    <div className="graph">
      <h1 className="vtitle">Knowledge graph</h1>
      <div className="graph-toolbar">
        <p className="vsub">
          Pressed 디지트와 관계 &middot; nodes {model.nodes.length} · links {visibleEdgeCount}
          {hiddenEdgeCount ? ` · hidden ${hiddenEdgeCount}` : ""}
        </p>
        <button type="button" className="graph-reset" onClick={resetView} title="Reset view">
          Reset
        </button>
      </div>
      <div className="gr">
        <GraphCanvas
          width={W}
          height={H}
          svgRef={svgRef}
          model={model}
          layout={layout}
          layoutById={layoutById}
          transform={transform}
          hoverId={hoverId}
          focusIds={focusIds}
          selectedId={selected?.id || null}
          visited={visited}
          onNodeEnter={setHoverId}
          onNodeLeave={() => setHoverId(null)}
          onNodePointerDown={beginDrag}
          onNodePointerMove={moveDrag}
          onNodePointerUp={endDrag}
          onNodePointerCancel={endDrag}
        />
        <GraphDetailsPanel selected={selected} hiddenEdgeCount={hiddenEdgeCount} />
      </div>
    </div>
  );
}
