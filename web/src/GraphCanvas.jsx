import React from "react";
import { buildDirectedEdgeGeometry } from "./graphGeometry.js";
import { graphNodeRadius } from "./graphPhysics.js";
import { openLeaf } from "./routes.js";

function endpointId(endpoint) {
  return typeof endpoint === "string" ? endpoint : endpoint?.id;
}

function resolveEndpoint(endpoint, layoutById, model) {
  if (typeof endpoint !== "string") return endpoint;
  return layoutById.get(endpoint) || model.nodeById.get(endpoint);
}

function shortText(value, max = 24) {
  if (!value) return "";
  return value.length > max ? `${value.slice(0, max - 1)}…` : value;
}

function stopGraphPan(event) {
  event.stopPropagation();
  event.nativeEvent.stopImmediatePropagation();
}

function GraphMarkers() {
  return (
    <defs>
      <marker id="leafGraphArrow" viewBox="0 0 10 10" markerWidth="9" markerHeight="9" refX="9" refY="5" orient="auto">
        <path d="M0,0 L10,5 L0,10 z" />
      </marker>
      <marker id="leafGraphArrowActive" viewBox="0 0 10 10" markerWidth="9" markerHeight="9" refX="9" refY="5" orient="auto">
        <path d="M0,0 L10,5 L0,10 z" />
      </marker>
    </defs>
  );
}

function GraphEdge({ link, layoutById, model, hoverId, linkCount }) {
  const source = resolveEndpoint(link.source, layoutById, model);
  const target = resolveEndpoint(link.target, layoutById, model);
  if (!source || !target) return null;

  const active = !hoverId || endpointId(link.source) === hoverId || endpointId(link.target) === hoverId;
  const edge = buildDirectedEdgeGeometry(source, target, {
    sourceRadius: graphNodeRadius(source),
    targetRadius: graphNodeRadius(target),
  });
  const showLabel = active && (hoverId || linkCount <= 26);
  const edgeClass = hoverId ? `edge${active ? " active" : " dim"}` : "edge";
  const marker = hoverId && active ? "url(#leafGraphArrowActive)" : "url(#leafGraphArrow)";

  return (
    <g>
      <path className={edgeClass} d={edge.path} markerEnd={marker} />
      {showLabel ? (
        <text className={`edge-label${hoverId && active ? " active" : ""}`} x={edge.label.x} y={edge.label.y} textAnchor="middle">
          {shortText(link.predicate, 18)}
        </text>
      ) : null}
    </g>
  );
}

function GraphNode({
  node,
  width,
  height,
  active,
  selected,
  visited,
  onNodeEnter,
  onNodeLeave,
  onNodePointerDown,
  onNodePointerMove,
  onNodePointerUp,
  onNodePointerCancel,
}) {
  const radius = graphNodeRadius(node);
  const classes = ["graph-node", active ? "" : "dim", selected ? "sel" : "", visited ? "visited" : ""].filter(Boolean).join(" ");

  return (
    <g
      className={classes}
      transform={`translate(${node.x || width / 2},${node.y || height / 2})`}
      onMouseEnter={() => onNodeEnter(node.id)}
      onMouseLeave={onNodeLeave}
      onMouseDownCapture={stopGraphPan}
      onTouchStartCapture={stopGraphPan}
      onPointerDown={(event) => onNodePointerDown(event, node)}
      onPointerMove={onNodePointerMove}
      onPointerUp={(event) => onNodePointerUp(event, node)}
      onPointerCancel={(event) => onNodePointerCancel(event, node)}
      onDoubleClick={() => openLeaf(node.slug)}
    >
      <circle className="node-halo" r={radius + 5} />
      <circle className="node-dot" r={radius} />
      <text className="graph-label" y={radius + 16} textAnchor="middle">
        {shortText(node.title)}
      </text>
    </g>
  );
}

export default function GraphCanvas({
  width,
  height,
  svgRef,
  model,
  layout,
  layoutById,
  transform,
  hoverId,
  focusIds,
  selectedId,
  visited,
  onNodeEnter,
  onNodeLeave,
  onNodePointerDown,
  onNodePointerMove,
  onNodePointerUp,
  onNodePointerCancel,
}) {
  return (
    <div className="canvas force">
      <svg ref={svgRef} className="graph-svg" viewBox={`0 0 ${width} ${height}`} width="100%" height={height}>
        <GraphMarkers />
        <rect className="graph-bg" width={width} height={height} />
        {model.nodes.length ? (
          <g className="graph-viewport" transform={transform.toString()}>
            <g className="edges">
              {layout.links.map((link, index) => (
                <GraphEdge
                  key={`${endpointId(link.source)}-${endpointId(link.target)}-${link.predicate}-${index}`}
                  link={link}
                  layoutById={layoutById}
                  model={model}
                  hoverId={hoverId}
                  linkCount={layout.links.length}
                />
              ))}
            </g>
            <g className="nodes">
              {layout.nodes.map((node) => (
                <GraphNode
                  key={node.id}
                  node={node}
                  width={width}
                  height={height}
                  active={!focusIds || focusIds.has(node.id)}
                  selected={selectedId === node.id}
                  visited={visited.has(node.id)}
                  onNodeEnter={onNodeEnter}
                  onNodeLeave={onNodeLeave}
                  onNodePointerDown={onNodePointerDown}
                  onNodePointerMove={onNodePointerMove}
                  onNodePointerUp={onNodePointerUp}
                  onNodePointerCancel={onNodePointerCancel}
                />
              ))}
            </g>
          </g>
        ) : (
          <text className="graph-empty" x={width / 2} y={height / 2} textAnchor="middle">
            pressed leaf graph가 비어 있습니다
          </text>
        )}
      </svg>
    </div>
  );
}
