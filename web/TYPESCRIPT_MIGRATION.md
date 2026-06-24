# TypeScript Migration

## Current Baseline

`npm run typecheck` runs TypeScript in no-emit readiness mode over the current JS/JSX source.

The baseline intentionally uses:

- `allowJs: true`
- `checkJs: false`
- `noEmit: true`
- React 18 type packages

This proves TypeScript can load the project, JSX runtime, Vite config, Node test imports, and D3 type packages without forcing a broad annotation pass yet.

## Deferred Strictness

`checkJs` is not enabled as a blocking gate yet. A probe with `allowJs + checkJs + noEmit` found broad follow-up work:

- implicit parameter and destructured prop types across React components and graph helpers
- DOM ref and pointer event types in graph interaction code
- graph node/link data model types
- D3 force/selection/zoom integration types
- CSS side-effect import declaration details

Those are real migration tasks, not setup tasks. Enforcing them here would turn the readiness baseline into a large source conversion.

## Migration Order

1. Type pure graph helpers first: `graphGeometry`, `graphPhysics`, `graphZoom`, and their tests.
2. Type graph data contracts: graph API payloads, nodes, links, and model output.
3. Type leaf UI components with explicit props: `GraphDetailsPanel`, `GraphCanvas`, `WorkspaceList`, and `ReviewReader`.
4. Type DOM refs and event handlers in graph interaction code.
5. Type API and route helpers, then `App`.
6. Rename files from `.js`/`.jsx` to `.ts`/`.tsx` only after their local contracts are explicit.

## Promotion Gate

Enable `checkJs` or begin file renames only when the first three migration slices have landed and `npm run typecheck` remains green.
