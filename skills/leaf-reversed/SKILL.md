---
name: leaf-reversed
description: Use when a finished artifact, code change, document, design, report, plan, model, or decision needs to be reverse-engineered into a complete LEAF record. Trigger on "reverse LEAF", "leaf-reversed", reconstructing Learn/Example/Architect/Feedback from an existing result, turning completed work into a citable `.leaf` leaf, or asking what LEAF gates would have contained.
---

# LEAF Reversed

Reverse LEAF starts from the result and reconstructs the LEAF that should have
existed. Produce a complete, evidence-based LEAF shape; do not write a fictional
process history.

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
3. Fill the gates in reverse-informed order, then clean them into normal LEAF
   reports:
   - **Learn:** inferred request, purpose, scope, facts, assumptions, unknowns.
   - **Example:** criteria the result appears to satisfy; one inspectable
     instance or decisive example visible in the result.
   - **Architect:** structure, design choices, task graph, and execution evidence
     implied by the result.
   - **Feedback:** review checks, residual risks, decisions needing user review,
     and lessons worth carrying forward.
4. Invoke `leaf-clean` before asking the user to review the reconstructed LEAF.

## Evidence Rules

Every reconstructed claim must carry one of these statuses:

| Status | Meaning |
|---|---|
| `Verified` | Directly visible in the artifact, source, command output, or user input. |
| `Inferred` | Strongly implied by repeated structure, naming, tests, constraints, or omissions. |
| `USER REVIEW NEEDED` | Author intent, priority, tradeoff, or external fact that the result alone cannot prove. |

Do not invent chronology, meetings, motivations, rejected alternatives, or user
approval. Write "the LEAF would record..." instead of "the team decided..." when
the result is the only evidence.

## Done Shape

A reversed LEAF is complete when `00-status.md` and the relevant gate files tell
a future reader what the result is, why it exists, what evidence supports it,
what remains uncertain, and what should be reused. If the user only asks for an
inline answer, use the same Learn / Example / Architect / Feedback headings.
