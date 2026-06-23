const DEFAULT_MIN_SCALE = 0.45;
const DEFAULT_MAX_SCALE = 3.4;

function clamp(value, min, max) {
  return Math.min(Math.max(value, min), max);
}

function wheelDelta({ deltaY, deltaMode = 0, ctrlKey = false }) {
  const modeFactor = deltaMode === 1 ? 0.05 : deltaMode ? 1 : 0.002;
  return -deltaY * modeFactor * (ctrlKey ? 10 : 1);
}

export function nextWheelZoom(
  current,
  point,
  { deltaY, deltaMode = 0, ctrlKey = false, minScale = DEFAULT_MIN_SCALE, maxScale = DEFAULT_MAX_SCALE } = {},
) {
  const k = clamp(current.k * Math.pow(2, wheelDelta({ deltaY, deltaMode, ctrlKey })), minScale, maxScale);

  if (k === current.k) {
    return { x: current.x, y: current.y, k: current.k, changed: false };
  }

  const ratio = k / current.k;
  return {
    x: point.x - (point.x - current.x) * ratio,
    y: point.y - (point.y - current.y) * ratio,
    k,
    changed: true,
  };
}
