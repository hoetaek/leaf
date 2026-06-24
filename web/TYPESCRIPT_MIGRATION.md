# TypeScript Migration

## Current Baseline

`npm run typecheck` runs TypeScript in strict no-emit mode over the `web/src` TypeScript source.

The web UI source now uses:

- `.ts` for logic, hooks, and tests without JSX
- `.tsx` for React components and component tests
- `allowJs: false`
- `strict: true`
- `noEmit: true`
- React 18, Vite, Node, and D3 type packages
- `typescript-eslint` for `.ts`/`.tsx` linting

This proves TypeScript can load the project, JSX runtime, DOM refs, graph interaction handlers, D3 force/selection/zoom integrations, and API data contracts without a JS fallback.

## Runtime Contracts

The conversion landed the following source contracts:

- `types.ts` defines graph API payloads, graph model/layout nodes and links, workspace list payloads, and review reader payloads.
- Graph force helpers type the D3 mutation boundary where links can start as ids and become node objects.
- Graph zoom and drag hooks type SVG refs, wheel events, pointer events, and simulation refs.
- Reader and workspace components type their DOM refs, keyboard handlers, stage filters, and response data.
- Storage helpers persist only string leaf ids.

## Test Gate

The test runner is unified on Vitest because Node's built-in test runner does not execute TypeScript source directly in this app setup.

The migration includes `src/typescriptMigration.test.ts`, which fails if `.js`, `.jsx`, `.mjs`, or `.cjs` files reappear under `web/src`.

## Maintenance Gate

Use `npm run check` before merging TypeScript changes. It runs lint, stylelint, Prettier check, unit tests, component tests, strict typecheck, and production build.
