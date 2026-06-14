---
name: leaf-reversed
description: Use when a finished artifact, code change, document, design, report, plan, model, or decision needs to be reverse-engineered into a complete LEAF record. Trigger on "reverse LEAF", "leaf-reversed", reconstructing Learn/Example/Architect/Feedback from an existing result, turning completed work into a citable `.leaf` leaf, or asking what LEAF gates would have contained.
---

# LEAF Reversed

Reverse LEAF starts from the result and reconstructs the LEAF that should have
existed. Produce a complete, evidence-based LEAF shape; do not write a fictional
process history.

## Standpoint

Stand at the author's desk **before the work existed**, not at a reader's chair
after it shipped. The job is to reverse-estimate the LEAF the author would have
written to produce this result — the raw words, why, and what they would have
set before touching the code — not to describe the finished artifact back to a
reader.

Across every gate the voice is author-prospective: "the author would have framed
the request as...", "작성자가 잡았을 의도는...". Never "this PR is...", "I
understand this as...", or any phrasing that only makes sense once the result is
already in hand.

The artifact is your evidence, not your subject. It proves what was *built*; it
does not prove what the author *set out to do*. Every reconstructed intent,
criterion, and design choice is therefore an inference resting on an artifact
footprint — keep the two layers separated (see Evidence Rules).

## Read First

- `../leaf-soul/SKILL.md` always.
- `../leaf-work/references/layout.md` before writing files.
- `../leaf-idea/references/gate-01-intent.md`,
  `../leaf-idea/references/gate-02-unknowns-context.md`, and
  `../leaf-work/references/gates.md` when judging gate content.

## Workflow

1. Inspect the finished artifact and its surrounding context: files, diff,
   rendered output, notes, tests, sources, or user-provided material.
2. Create or resume one matching LEAF folder. Completed reconstructions belong
   under `.leaf/02-leaves/<slug>/`; use a sprout only while investigation is
   still incomplete.
3. Fill the gates in reverse-informed order, each reconstructed from the
   author's pre-work standpoint, then clean them into normal LEAF reports:
   - **Learn:** the request, why, and what the author would have set before
     touching the code — inferred from footprints, never the shipped text
     restated as fact. The Intent gate (Raw wording, Why, Provisional what,
     Locked intent) is author-prospective and inferred end to end.
   - **Example:** the criteria the author would have had to satisfy, and the one
     instance in the result that would have proven them.
   - **Architect:** the structure, design choices, and task slicing the author
     would have planned, as implied by the result.
   - **Feedback:** the review checks, residual risks, decisions needing user
     review, and lessons the author would have recorded.
4. Invoke `leaf-clean` before asking the user to review the reconstructed LEAF.

## Evidence Rules

Separate two layers in every claim:

- **Footprint** — what the artifact, diff, test, source, or command output
  directly shows. This is the `Verified` evidence.
- **Author intent** — the raw words, why, what, criterion, or design rationale
  the author would have set up to leave that footprint. This is `Inferred` (or
  `USER REVIEW NEEDED`). A footprint never proves the intent behind it; cite the
  footprint as the evidence the inference rests on.

| Status | Meaning |
|---|---|
| `Verified` | A footprint directly visible in the artifact, source, command output, or user input. |
| `Inferred` | An author choice — intent, criterion, rationale — strongly implied by a footprint but not proven by it. |
| `USER REVIEW NEEDED` | Author intent, priority, or tradeoff that no footprint constrains enough to infer. |

Because the reconstruction is author-prospective, the Intent gate (Raw wording,
Why, Provisional what, Locked intent) is `Inferred` end to end: the shipped PR
text is a footprint, not the author's original raw words, so do not stamp it
`Verified` just because it is visible.

Do not invent chronology, meetings, motivations, rejected alternatives, or user
approval. Write "the author would have recorded..." instead of "the team
decided..." when the result is the only evidence.

## Done Shape

A reversed LEAF is complete when `00-status.md` and the relevant gate files let a
future reader see the LEAF the author would have written to reach this result:
the raw words and why they would have started from, the what and criteria they
would have aimed at, the footprints that ground each inference, and what no
footprint can settle. If the user only asks for an inline answer, use the same
Learn / Example / Architect / Feedback headings and the same author-prospective
voice.
