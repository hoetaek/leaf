import { leafHref } from "./routes";
import { useJsonResource } from "./useJsonResource";
import { WORKSPACE_GATES } from "./workspaceModel";
import { buildTreeViewModel, treeItemPhaseLabel, type TreeStageSummary, type TreeViewModel } from "./treeViewModel";
import type { GraphApiResponse, LeafStatus, WorkspaceListResponse, WorkspaceRow } from "./types";

function formatCount(value: number | null): string {
  return value === null ? "—" : String(value);
}

function GrowthSnapshot({ model }: { model: TreeViewModel }) {
  const foliage = [
    { cx: 78, cy: 42, rx: 17, ry: 11, rotate: -28 },
    { cx: 108, cy: 30, rx: 18, ry: 12, rotate: -8 },
    { cx: 138, cy: 45, rx: 17, ry: 11, rotate: 24 },
    { cx: 96, cy: 65, rx: 19, ry: 12, rotate: -18 },
    { cx: 126, cy: 64, rx: 20, ry: 13, rotate: 8 },
    { cx: 158, cy: 68, rx: 18, ry: 12, rotate: 22 },
    { cx: 65, cy: 72, rx: 15, ry: 10, rotate: -36 },
    { cx: 112, cy: 86, rx: 18, ry: 11, rotate: -10 },
    { cx: 145, cy: 88, rx: 19, ry: 12, rotate: 16 },
    { cx: 180, cy: 88, rx: 15, ry: 10, rotate: 32 },
  ];
  const filledFoliage = Math.min(foliage.length, Math.max(1, Math.ceil((model.counts.leaves || 0) / 4)));
  const fruitCount = Math.min(5, Math.max(0, Math.ceil((model.pressedCount || 0) / 5)));
  const pressedLabel = `${formatCount(model.pressedCount)} pressed`;

  return (
    <figure className="tree-snapshot" aria-label={`Tree snapshot, ${model.maturityLabel}`}>
      <svg viewBox="0 0 240 180" role="img" aria-labelledby="tree-snapshot-title">
        <title id="tree-snapshot-title">LEAF growth snapshot</title>
        <path className="tree-branch" d="M120 150 C118 118 119 94 120 66" />
        <path className="tree-branch thin" d="M120 94 C94 82 76 66 64 46" />
        <path className="tree-branch thin" d="M121 86 C145 72 160 58 174 38" />
        <path className="tree-branch thin" d="M120 112 C98 105 80 98 62 90" />
        <path className="tree-branch thin" d="M122 112 C150 106 172 100 190 92" />
        {foliage.slice(0, filledFoliage).map(({ cx, cy, rotate, rx, ry }) => (
          <ellipse
            key={`${cx}-${cy}`}
            className="tree-leaf"
            cx={cx}
            cy={cy}
            rx={rx}
            ry={ry}
            transform={`rotate(${rotate} ${cx} ${cy})`}
          />
        ))}
        {Array.from({ length: fruitCount }, (_, index) => (
          <circle key={index} className="tree-fruit" cx={88 + index * 22} cy={62 + (index % 2) * 24} r="5" />
        ))}
        <path className="tree-ground" d="M72 153 C96 162 145 162 168 153" />
      </svg>
      <figcaption>
        {model.maturityLabel} · {model.totalWork} work · {pressedLabel}
      </figcaption>
    </figure>
  );
}

function TreeMetric({ label, value, note }: { label: string; value: string | number; note: string }) {
  return (
    <div className="tree-metric">
      <span className="tree-metric-label">{label}</span>
      <strong>{value}</strong>
      <em className="tree-metric-note">{note}</em>
    </div>
  );
}

function MiniProgress({ status }: { status?: LeafStatus }) {
  const done = status?.progress_done || 0;
  const cur = status?.progress_current || 0;

  return (
    <span className="tree-mini-progress" aria-label={status?.progress_label || "progress unknown"}>
      {WORKSPACE_GATES.map((gate) => (
        <span key={gate} className={`tree-mini-segment${gate <= done ? " done" : gate === cur ? " cur" : ""}`} />
      ))}
    </span>
  );
}

function TreeRow({ row }: { row: WorkspaceRow }) {
  return (
    <li>
      <a className="tree-row" href={leafHref(row.slug)}>
        <span className="tree-row-main">
          <span className={`tagdot ${row._stage}`} />
          <b className="tree-row-slug">{row.slug}</b>
          <span className="tree-row-meta">{treeItemPhaseLabel(row)}</span>
        </span>
        <MiniProgress status={row.status} />
      </a>
    </li>
  );
}

function TreeStageSection({ stage }: { stage: TreeStageSummary }) {
  const defaultOpen = stage.key !== "fallen";

  return (
    <details className={`tree-stage ${stage.key}`} open={defaultOpen}>
      <summary>
        <span className="tree-stage-title">
          <span className={`tagdot ${stage.key}`} />
          <b className="tree-stage-name">{stage.label}</b>
        </span>
        <span className="tree-stage-count">
          {stage.count} · {stage.stateLabel}
        </span>
      </summary>
      {stage.items.length ? (
        <ul>
          {stage.items.map((row) => (
            <TreeRow key={`${stage.key}-${row.slug}`} row={row} />
          ))}
        </ul>
      ) : (
        <p className="tree-empty-stage">{stage.emptyLabel}</p>
      )}
    </details>
  );
}

export default function TreeView() {
  const workspace = useJsonResource<WorkspaceListResponse>("/api/list");
  const graph = useJsonResource<GraphApiResponse>("/api/graph");

  if (workspace.error) return <p className="err">Tree를 불러오지 못했습니다: {workspace.error}</p>;
  if (!workspace.data) return <p className="muted">불러오는 중…</p>;

  const model = buildTreeViewModel(workspace.data, graph.data);
  const graphNote = graph.error ? "unavailable" : "graph";

  return (
    <div className="tree">
      <h1 className="vtitle">Tree</h1>
      <p className="vsub">
        {model.counts.leaves || 0} leaves &middot; {model.counts.sprouts || 0} sprouts &middot;{" "}
        {model.counts.fallen || 0} fallen &middot; {formatCount(model.pressedCount)} pressed
      </p>

      <section className="tree-hero" aria-labelledby="tree-overview-title">
        <div>
          <h2 id="tree-overview-title" className="tree-heading">
            Lifecycle
          </h2>
          <div className="tree-metrics" aria-label="Tree metrics">
            <TreeMetric label="Leaves" value={model.counts.leaves || 0} note={model.maturityLabel} />
            <TreeMetric label="Sprouts" value={model.counts.sprouts || 0} note="active" />
            <TreeMetric label="Pressed" value={formatCount(model.pressedCount)} note={graphNote} />
            <TreeMetric label="Fallen" value={model.counts.fallen || 0} note="archived" />
            <TreeMetric label="Edges" value={formatCount(model.edgeCount)} note={graphNote} />
          </div>
        </div>
        <GrowthSnapshot model={model} />
      </section>

      <section className="tree-stage-grid" aria-label="Lifecycle stages">
        {model.stages.map((stage) => (
          <TreeStageSection key={stage.key} stage={stage} />
        ))}
      </section>

      <p className="foot-note">
        <span className="kbd">1</span> Workspace &middot; <span className="kbd">2</span> Graph &middot;{" "}
        <span className="kbd">3</span> Tree
      </p>
    </div>
  );
}
