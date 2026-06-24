import { useCallback, useEffect, useRef, useState } from "react";
import { select } from "d3-selection";
import { zoom, zoomIdentity } from "d3-zoom";
import type { D3ZoomEvent, ZoomBehavior, ZoomTransform } from "d3-zoom";
import { nextWheelZoom } from "./graphZoom";
import type { GraphModel, ZoomTransformState } from "./types";

interface GraphZoomOptions {
  minScale: number;
  maxScale: number;
}

interface PointerLike {
  clientX: number;
  clientY: number;
}

export function useGraphZoom(model: GraphModel | null, { minScale, maxScale }: GraphZoomOptions) {
  const svgRef = useRef<SVGSVGElement | null>(null);
  const zoomRef = useRef<ZoomBehavior<SVGSVGElement, unknown> | null>(null);
  const transformRef = useRef<ZoomTransform>(zoomIdentity);
  const [transform, setTransform] = useState<ZoomTransform>(zoomIdentity);

  const svgPoint = useCallback((event: PointerLike): [number, number] => {
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
    (event: PointerLike): [number, number] => {
      return transformRef.current.invert(svgPoint(event));
    },
    [svgPoint],
  );

  const applyZoomTransform = useCallback((next: ZoomTransformState) => {
    const nextTransform = zoomIdentity.translate(next.x, next.y).scale(next.k);
    if (svgRef.current && zoomRef.current) {
      select<SVGSVGElement, unknown>(svgRef.current).call(zoomRef.current.transform, nextTransform);
    } else {
      transformRef.current = nextTransform;
      setTransform(nextTransform);
    }
  }, []);

  const handleWheel = useCallback(
    (event: WheelEvent) => {
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
          minScale,
          maxScale,
        },
      );

      if (next.changed) {
        applyZoomTransform(next);
      }
    },
    [applyZoomTransform, maxScale, minScale, svgPoint],
  );

  useEffect(() => {
    if (!model || !svgRef.current) return undefined;

    const svg = select<SVGSVGElement, unknown>(svgRef.current);
    const zoomer = zoom<SVGSVGElement, unknown>()
      .scaleExtent([minScale, maxScale])
      .on("zoom", (event: D3ZoomEvent<SVGSVGElement, unknown>) => {
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
  }, [maxScale, minScale, model]);

  useEffect(() => {
    if (!model || !svgRef.current) return undefined;

    const svg = svgRef.current;
    svg.addEventListener("wheel", handleWheel, { passive: false });
    return () => {
      svg.removeEventListener("wheel", handleWheel);
    };
  }, [handleWheel, model]);

  const resetView = useCallback(() => {
    if (!svgRef.current || !zoomRef.current) return;
    select<SVGSVGElement, unknown>(svgRef.current).call(zoomRef.current.transform, zoomIdentity);
  }, []);

  return { svgRef, transform, graphPoint, resetView };
}
