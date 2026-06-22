---
name: split
description: |
  Use when deciding how to split one piece of work into separate leaves —
  whether to split at all, along which grain, and how the pieces stay linked.
  Trigger on `leaf split`, "split this", "should I split", "쪼갤까", "어느 결로
  쪼개", "한 작업을 나눈다", "decompose this", or a bundled sprout that may hold
  independent cores. This first increment carries the judgment framework; the
  execution (parent fall, child new, link wiring) is a later increment.
---

# LEAF Split

Splitting one work item is not one move. It is three layers, and the common
mistake is to cut along several grains at once. **A work item splits cleanly
along only one grain at a time** — pick one, and the grains you did not pick
scatter and tangle (the "tyranny of the dominant decomposition"). So this
skill's real job is not "cut it every way" but **"choose which coherence you are
willing to lose."**

The three layers:

| Layer | Question |
|---|---|
| **(a) whether / when** | Should this split now at all? |
| **(b) which grain** | If yes, which single dominant cut-axis? |
| **(c) order & link** | After the cut, how do the pieces order and stay linked? |

(a) and (b) are **co-determined**, not strictly sequential — seeing a clean grain
*is* evidence to split; seeing none *is* evidence to keep grouped. Only (c) is
strictly downstream: order and links only mean something after a cut exists.

## (a)+(b) — split now, and along which grain

Look at both together:

1. **Is there a clean grain?** Scan the menu below for an axis this work divides
   cleanly along.
   - A clear grain is visible → that visibility is the case for **split now**.
   - No clean grain is visible → that is the case for **keep grouped** (it is one
     work item). "No grain" is a valid terminal answer, not a failure to look
     harder.
2. **Cross-check with `learn`'s Split Check** (`split now / keep grouped /
   ask first`). Reuse that judgment — do not invent competing criteria.

### The cut-axis menu (open, not closed)

Pick the **single sharpest** grain. These are the common ones; if none fits,
name the sharpest cut yourself from the broader taxonomy.

| Grain | Signal that this is the cut |
|---|---|
| Responsibility | each piece has a different reason to change (SRP) |
| Abstraction level | concept/definition vs implementation/realization |
| Lifecycle / review path | outputs, tests, and review paths run separately |
| Volatility | a fast-changing part vs a stable part |
| Form vs content | the substance vs the shape it is delivered in |
| Feature / value | a vertical slice that delivers value on its own |
| Disposability | one piece can be dropped or deferred and the rest still lives |
| *other* | name the sharpest cut from your decomposition knowledge |

### Choosing the dominant grain

1. Take the **one** grain it divides along most cleanly (one axis per pass).
2. **Name the lost cohesion in one line** — the most painful thing the
   *unpicked* grains will scatter. The unpicked axes always scatter; say which
   loss hurts.
3. **Reject gate:** weigh the lost cohesion against the separation gained. If the
   loss hurts more than the separation helps, **drop that axis and try another**.
   (Splitting a feature by file-type/layer gains separation but loses feature
   cohesion → reject.) This step rejects bad axes, it does not just disclose them.
4. **Second axis also load-bearing?** Do not cut two ways at once. Split on the
   dominant axis now, then **re-enter this skill on a child** for the second cut
   (recursive split).
5. **Tie between axes?** Keep the one that loses the less-painful cohesion;
   record the other in the lost-cohesion line. If still undecidable, ask the user
   `which cut dominates?` — do not bounce this back to (a).

### When to stop / ask

| Outcome | Do |
|---|---|
| grain visible + split now | go to (c) |
| no grain / keep grouped | **stop.** Return "keep it as one work item" + the reason |
| ambiguous independence | ask the user `are these independent?` (an (a) question) |
| ambiguous grain | ask `which cut dominates?` (a (b) question) |

## (c) — order & link the pieces

Split the work into children (N ≥ 2). For each child record:

- **grain** — which side of the dominant cut it is.
- **lineage** — `split from: <parent>` (one direction, ancestry).
- **directional dependency (if any)** — "A blocked-by B". Write it as
  human-readable prose in the status file (`split reason:`, `source <X>:`), and
  also record the eventual graph edge it implies. Split itself still does not
  create `linked.md` before a child is pressed, because `linked.md` belongs next
  to a pressed digest. The split judgment should make the later edge obvious.

Use this mapping when a split relationship becomes pressed knowledge:

| Split relation | Later `linked.md` edge |
|---|---|
| child came from parent | `derived_from` -> `leaf:<parent-slug>` |
| child A is blocked by child B | in A: `depends_on` -> `leaf:<child-b-slug>` |
| sibling leaves are related but unordered | `related_to` -> `leaf:<sibling-slug>` |

**Order:** topologically sort by the directional dependencies → which child must
come first.

**Cycle?** A cycle means the cut was wrong, not that ordering is hard. Re-cut on
a different grain (back to (b)); if the pieces are genuinely mutually inseparable,
keep grouped / ship atomically (back to (a)).

## Over-decomposition guard (every level)

Before emitting children, and **at every recursion level**:

- "Can any child be dropped or deferred?" If **no**, the split is wrong → fall
  back to keep grouped.
- Premature-decomposition pressure is real (the microservices rebound): when in
  doubt, do not split — keep or ask.
- **Recursion terminates** when a node is atomic (its pieces cannot be deferred)
  or the second axis is no longer load-bearing in that child. Stop descending
  there.

## Output

- verdict: split now / keep grouped / ask
- if split: the dominant grain + the lost-cohesion line
- child list: each child's grain, lineage, directional dependency
- link plan: future `linked.md` rows for pressed children, using only
  `derived_from`, `depends_on`, or `related_to`
- order: the topological sequence
- the over-decomposition guard result

## Boundaries

- **Reuse `learn`'s Split Check for (a)** — do not author new split criteria here.
- **Execution is a later increment.** This skill produces the *judgment*; the
  parent `leaf fall --reason split`, child `leaf new`, and link wiring are not
  automated yet. The keep/press/fall actions and the `split` fallen reason live
  in `using-leaf` ("Ending a leaf").
- **Do not create `linked.md` early.** It belongs beside `pressed.md` in
  `.leaf/02-leaves/<slug>/` after the relevant child becomes citable. Before
  then, keep the link plan in the split output/status prose.
- Conduct, voice, and language come from `soul`.

## Worked check

Reverse-validate against a real split: `folder-tree-visualization` bundled tree
rendering + link-edge recording; the recording was split off as
`citation-link-recording`. Running the framework: (a)+(b) a clean
**responsibility** grain is visible (rendering vs edge data) → split now;
lost cohesion = "seeing links right inside the tree"; (c) the tree is
*blocked-by* the link model, so the link model comes first. That reproduces the
decision actually made — which is the bar this framework must clear.
