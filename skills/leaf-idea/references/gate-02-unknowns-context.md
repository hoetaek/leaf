# ② Unknowns & Context

**What ② builds: you, the eager learner, come to genuinely understand this — from
knowing the terms to being able to judge it for yourself.** Learning runs as a
trajectory: from coming to know domain knowledge, conventions, related existing
work, comparables, prior art, and whether this is even the only way, to being
able to judge what to choose between and on what basis. Both ends matter, and
neither is in a hurry; depth is welcome here.

Be the explorer-companion. Do not only file the unknowns the user names. First
map what is worth learning in this topic — domain concepts, history,
conventions, counter-examples, live debates, surprising connections to other
fields — and offer it as a menu, surfacing threads the user did not know to ask
about. Then follow the user's curiosity: go deep on the thread they pick,
amplify it, and connect it onward. The menu opens the space wider; it never
narrows it.

The experiment at this gate aims at the world: is this true? Verify what can be
verified rather than believing it on trust — verification is itself the deepest
kind of learning.

Name what is missing, then answer those entries in the same working file.
Unknown surfacing and reference exploration are one loop: write the question,
search or ask for the answer, then update that same entry with what was found.

Before searching, benchmarking, or researching anything, create the first pass of
unknowns. Without this, exploration becomes reactive.

Use the Clarity Ledger as a lens here, not a checklist to fill in: it only helps
point learning at the gap that matters most right now.

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
the after architecture; deciding the future structure is a different kind of
work, not learning.

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
- **Unresolved**: kept as an explicit open question, returned to later, or
  deferred with a reason.

## Premise Inventory

② is not only a context inventory. It must expose the premises you are taking for
granted — the things assumed true without ever looking. Naming them is one of the
deepest learning moves: what looks obvious is often where understanding is
thinnest.

For each judgment you find yourself leaning on, write:

- the judgment;
- why it matters;
- the premises required for it to be true;
- the source or evidence for each premise;
- the status of each premise: `FACT`, `VERIFIED`, `ASSUMPTION`, or
  `USER REVIEW NEEDED`;
- what you would believe instead if that premise turned out false.

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

| Judgment | Required premise | Evidence / Source | Status | If false, what changes |
|---|---|---|---|---|
| ... | ... | ... | FACT / VERIFIED / ASSUMPTION / USER REVIEW NEEDED | ... |
```

Reach for the premise table whenever you catch yourself treating something as
settled that you have not actually checked — a definition everyone "knows", a
convention assumed universal, a claim repeated without a source, or the surface
wording taken as obviously true. These are exactly the spots where an eager
learner digs in instead of nodding along.

Do not let Learn rest while an `ASSUMPTION` you are leaning on stays unchecked,
unless it is explicitly marked for user review or a cheap verification path is
recorded. Verify it, ask the user, or write down plainly what you would believe
instead if it were false.

Record Gate ② experiments as `hypothesis -> test -> result`. They target the
world, convention, repo, source material, or audience condition: "is this true?"
Do not use ② experiments to validate a proposed answer shape; ② tests the world,
not your own answer.

Gate to continue:

- Unknowns are grouped by category, not one flat list.
- Each is marked "blocking now" vs "useful later".
- The most expensive unknowns, the ones that would unravel later work, are
  identified.
- Blocking unknowns have sourced answers, explicit assumptions, or owner/user
  questions.
- The fact/assumption boundary is visible.
- The judgments the user is leaning on are named, with the premises under them
  surfaced.
- For structurally complex or brownfield work, a current-state / problem-space
  ASCII map exists under `01-Learn/02-references/`, or the reason it was not
  needed is stated.
- The map labels facts, `UNKNOWN`s, and `ASSUMPTION`s clearly enough to expose
  missing context, split signals, and candidate choices without drifting into
  after-design.
- The premises required for each key judgment are listed with a source or cheap
  verification path and one of `FACT`, `VERIFIED`, `ASSUMPTION`, or
  `USER REVIEW NEEDED`.
- Any `ASSUMPTION` the user is leaning on is resolved, explicitly escalated for
  user review, or recorded with what would be believed instead if it were false.
- Any change from ①'s provisional what is recorded with the fact or premise that
  caused the change.
- The discovery is deep enough that the user genuinely understands the terrain —
  not padded with collection for its own sake, but never cut short while a thread
  still pulls.
- 2-4 plausible directions or frames are named when the user needs references to
  picture the landscape.
- The user can say what to choose between and on what basis; learning has reached
  judgment, not just collection.

Before resting Learn, always ask the user this explicitly — and show the
`01-Learn/02-references/` file tree first, with a one-line note per file, so the
user judges from the gathered files rather than from memory. An empty or thin
tree must be named plainly, with the reason references were not needed:

> 알고 싶던 걸 충분히 알게 됐나요? 아직 당신을 끌어당기는 결 — 더 파고 싶은
> 개념, 보고 싶은 사례, 짚어보고 싶은 논쟁, 확인하지 않은 가정 — 이 남아
> 있나요?

Return here whenever a new unknown surfaces, a thread you set aside starts
pulling again, or later work reveals a prior assumption was wrong.
`01-Learn/02-unknowns.md` evolves throughout.
