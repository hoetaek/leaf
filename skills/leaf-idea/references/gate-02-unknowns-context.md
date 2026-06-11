# ② Unknowns & Context

**What ② builds: you can judge what this needs, having learned it yourself.**
Learning runs as a trajectory: from coming to know domain knowledge,
conventions, related existing code, comparables, prior art, and whether this is
even the only way, to being able to judge what to choose between and on what
basis. The gate closes only at that second point, and that point is exactly what
③ Criteria consumes.

The experiment at this gate aims at the world: is this true? Verify what can be
verified before any answer is built. ④ Wireframe later aims the same scrutiny at
your own answer instead.

Name what is missing, then answer those entries in the same working file.
Unknown surfacing and reference exploration are one loop: write the question,
search or ask for the answer, then update that same entry with what was found.

Before searching, benchmarking, or researching anything, create the first pass of
unknowns. Without this, exploration becomes reactive.

Use the Clarity Ledger as a lens here, not a checklist to fill in. The ledger is
scored and locked at ③, not here; ② only uses it to point learning at the right
gap.

Categorize unknowns along two axes: what kind of knowledge, and from where.

By kind:

- **Domain concepts**: meanings of core terms used by the audience or judge.
- **Standards / conventions**: accepted tone, format, structural patterns, and
  best practices for this output form.
- **Selection criteria**: what to choose between, the trade-offs in play, and the
  principle that decides. This conditional knowledge is what turns learning into
  judgment.

By source:

- **External**: comparable cases, prior art, benchmark examples, authoritative
  sources, recent domain changes.
- **Internal**: what the team already has, such as data, footage, contacts,
  documents, prior decisions, and forgotten constraints.

## Big-Picture Map

For structurally complex or brownfield work, learn by drawing the current shape.
Create a compact ASCII map under `01-Learn/02-references/` when the work touches
an existing system, product architecture, workflow, domain model, multiple
actors/surfaces, or a reference set large enough that the shape is easy to lose.

The map is a study artifact, not a design artifact. It should show the current
known topology and problem space: actors, surfaces, components, data/content
flow, ownership boundaries, external dependencies, preserved constraints, and
gaps marked `UNKNOWN` or `ASSUMPTION`. It may compare reference models or
existing alternatives when that helps judgment, but it must not silently choose
the after architecture; ⑤ Design owns the generator and the future structure.

Keep the map cheap and inspectable. ASCII boxes/arrows, a small state-flow, or a
component table is enough. Summarize the learned facts, split signals, candidate
choices, and unresolved gaps back into `02-unknowns.md`; do not make later gates
read a diagram to discover the facts.

Search the web actively for convention and external items when the domain can
shift over time. Save what you find under `01-Learn/02-references/`, one file per
topic, and summarize the useful answer back in `02-unknowns.md`.

Ask or infer:

- What term, standard, fact, or internal context would I currently have to guess?
- Which guess, if wrong, would unravel later work?
- Which unknowns block the next gate, and which are useful-later only?

Resolve by updating the same entries:

- **Verified fact**: what is now known, with a source or direct user-provided
  basis.
- **Flagged assumption**: what is still being treated as true without proof;
  mark which assumptions would unravel downstream work.
- **Inventoried material / condition**: concrete resources and constraints
  downstream gates will rely on.
- **Unresolved**: carried to ③ as an explicit assumption, returned to this gate
  later, or deferred with a reason.

## Premise Inventory

② is not only a context inventory. It must expose the premises that later
criteria and design judgments depend on.

For each candidate judgment that would shape ③ Criteria or ⑤ Design, write:

- the candidate judgment;
- why that judgment matters;
- the premises required for it to be true;
- the source or evidence for each premise;
- the status of each premise: `FACT`, `VERIFIED`, `ASSUMPTION`, or
  `USER REVIEW NEEDED`;
- the smaller alternative if that premise turns out false.

Use these status labels:

| Label | Meaning |
|---|---|
| `FACT` | Already confirmed by code, documents, user confirmation, official material, or another available source |
| `VERIFIED` | Directly checked during this Learn pass |
| `ASSUMPTION` | Reasonable but not yet verified |
| `USER REVIEW NEEDED` | Only the user can confirm it, or the user must decide it |

Use this shape in `01-Learn/02-unknowns.md`:

```md
## Premise Inventory

### Candidate judgments

| Candidate judgment | Why this judgment matters | Status |
|---|---|---|
| ... | ... | FACT / VERIFIED / ASSUMPTION / USER REVIEW NEEDED |

### Required premises

| Judgment | Required premise | Evidence / Source | Status | If false, simpler alternative |
|---|---|---|---|---|
| ... | ... | ... | FACT / VERIFIED / ASSUMPTION / USER REVIEW NEEDED | ... |
```

Use the premise table when the work may add a new abstraction, service, action,
query, DTO, schema, workflow, UI/API payload, or policy calculation; remove an
existing condition and replace it with a new calculation rule; reimplement
something already guaranteed by an existing lifecycle or state transition; grow
a payload or surface area; or treat the raw wording's what as the implementation
direction.

Do not close ② if an `ASSUMPTION` directly causes the design to become larger,
unless it is explicitly marked for user review or a cheap verification path is
recorded. First verify the premise, ask the user, or carry a smaller alternative
into ③.

Record Gate ② experiments as `hypothesis -> test -> result`. They target the
world, convention, repo, source material, or audience condition before an answer
is built: "is this true?" Do not use ② experiments to validate a proposed answer
shape; that scrutiny belongs to ④.

Gate to continue:

- Unknowns are grouped by category, not one flat list.
- Each is marked "blocking now" vs "useful later".
- The most expensive unknowns, the ones that would unravel later work, are
  identified.
- Blocking unknowns have sourced answers, explicit assumptions, or owner/user
  questions.
- The fact/assumption boundary is visible.
- The key candidate judgments for ③ Criteria or ⑤ Design are named.
- For structurally complex or brownfield work, a current-state / problem-space
  ASCII map exists under `01-Learn/02-references/`, or the reason it was not
  needed is stated.
- The map labels facts, `UNKNOWN`s, and `ASSUMPTION`s clearly enough to expose
  missing context, split signals, and candidate choices without drifting into
  after-design.
- The premises required for each key judgment are listed with a source or cheap
  verification path and one of `FACT`, `VERIFIED`, `ASSUMPTION`, or
  `USER REVIEW NEEDED`.
- Any `ASSUMPTION` that would increase design size is resolved, explicitly
  escalated for user review, or paired with a smaller alternative.
- Any change from ①'s provisional what is recorded with the fact or premise that
  caused the change.
- The discovery set is bounded enough for the current decision, not exhaustive.
- 2-4 plausible directions or frames are named when the user needs references to
  picture the criteria.
- The user can say what to choose between and on what basis; learning has reached
  judgment, not just collection.

Before closing Learn, always ask the user this explicitly:

> 정말로 이 일을 판단하고 기준화하기 위해 배우고 알아야 하는 사실을 다
> 알았나요? 아직 확인해야 할 사실, 참고해야 할 사례, 내부 맥락, 또는
> 검증하지 않은 가정이 남아 있나요?

Return here when a new unknown surfaces mid-work, when research detours start
interrupting drafting, or when a downstream gate reveals a prior assumption was
wrong. `01-Learn/02-unknowns.md` evolves throughout.
