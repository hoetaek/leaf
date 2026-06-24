import { useCallback, useMemo, useState } from "react";
import GraphCanvas from "./GraphCanvas";
import GraphDetailsPanel from "./GraphDetailsPanel";
import { GRAPH_VIEWPORT } from "./graphLayout";
import { buildGraphModel } from "./graphModel";
import { useForceGraphLayout } from "./useForceGraphLayout";
import { useGraphNodeDrag } from "./useGraphNodeDrag";
import { useGraphZoom } from "./useGraphZoom";
import { useJsonResource } from "./useJsonResource";
import { useVisitedLeaves } from "./useVisitedLeaves";
import type { GraphApiResponse, GraphLayoutNode } from "./types";

const {
  width: W,
  height: H,
  boundsPadding: GRAPH_BOUNDS_PADDING,
  minZoom: MIN_ZOOM,
  maxZoom: MAX_ZOOM,
} = GRAPH_VIEWPORT;

export default function GraphView() {
  const { data, error } = useJsonResource<GraphApiResponse>("/api/graph");
  const [selId, setSelId] = useState<string | null>(null);
  const [hoverId, setHoverId] = useState<string | null>(null);

  const model = useMemo(() => (data ? buildGraphModel(data) : null), [data]);

  const effectiveSelId = selId && model?.nodeById.has(selId) ? selId : model?.nodes[0]?.id || null;
  const selected = effectiveSelId && model ? (model.nodeById.get(effectiveSelId) ?? null) : null;
  const focusIds = useMemo(() => {
    if (!hoverId || !model) return null;
    return model.neighboursById.get(hoverId) || null;
  }, [hoverId, model]);
  const visibleEdgeCount = model?.links.length || 0;
  const hiddenEdgeCount = data && model ? Math.max(0, (data.edges?.length ?? 0) - model.links.length) : 0;
  const { layout, nodeRef, simulationRef } = useForceGraphLayout(model, {
    width: W,
    height: H,
    boundsPadding: GRAPH_BOUNDS_PADDING,
  });
  const { svgRef, transform, graphPoint, resetView } = useGraphZoom(model, {
    minScale: MIN_ZOOM,
    maxScale: MAX_ZOOM,
  });
  const { visited, markVisited } = useVisitedLeaves();

  const selectNode = useCallback(
    (node: GraphLayoutNode) => {
      if (!node?.id) return;
      setSelId(node.id);
      markVisited(node.id);
    },
    [markVisited],
  );

  const layoutById = useMemo(() => new Map(layout.nodes.map((node) => [node.id, node])), [layout.nodes]);
  const { beginDrag, moveDrag, endDrag } = useGraphNodeDrag({
    width: W,
    height: H,
    boundsPadding: GRAPH_BOUNDS_PADDING,
    graphPoint,
    nodeRef,
    simulationRef,
    onSelectNode: selectNode,
  });

  if (error) return <p className="err">그래프를 불러오지 못했습니다: {error}</p>;
  if (!data || !model) return <p className="muted">불러오는 중…</p>;

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
