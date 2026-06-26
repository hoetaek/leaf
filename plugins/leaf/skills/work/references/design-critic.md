# Critic (Design Falsification)

Gate ⑥, between ⑤ Design and ⑦ Tasks. ⑤ generalized one validated instance into
a generator; this gate tries to *falsify* that generalization before ⑦ Tasks
builds on it — the same scrutiny ④ gave ③, now aimed at ⑤. **It always runs;
what scales with risk is the depth.** It is not an automatic consensus loop and
does not require a specific agent runtime. The reviewer may be the user, another
human, another agent, or a subagent.

## How deep to go

Every design gets at least a quick self-pass. A low-risk quick self-pass still
leaves a one-line record of what was checked — preferably in
`03-Architect/05-design.md` when no `06-critic.md` file is created. Go deep —
external reviewer, multiple lenses, recorded rationale — when the design
involves any of:

- security, privacy, safety, compliance, or permission boundaries
- migrations, destructive changes, data loss risk, or irreversible operations
- public terminology, interface, API, policy, workflow, or document structure
- new names, structures, locations, taxonomies, formats, examples, or policy
  language that future work may copy as precedent
- cross-team, cross-module, or cross-artifact coupling
- large user-facing behavior, argument, narrative, or visual shifts
- one asserted option with weak alternatives or unclear decision drivers

## Verdicts

Use one verdict:

- `APPROVE`: the design is actionable without guessing.
- `ITERATE`: the design is promising but needs specific revision before ⑦ Tasks.
- `REJECT`: the design is not safe or coherent enough to task.

## Review Criteria

Check:

- Intent fit: the design is the best available answer to the necessity / why
  locked at ① Intent, not only a plausible way to satisfy the current ③
  Criteria. If a simpler, cheaper, or more direct design would resolve the why,
  require iteration or record why it was rejected.
- Principles and drivers: the chosen option follows the stated principles and
  decision drivers.
- Fair alternatives: viable options are real, not strawmen. If only one option
  remains, rejected options have explicit invalidation rationale.
- Steelman antithesis: the strongest argument against the chosen option is
  stated and answered.
- Requirements fit: the design covers the active Clarity Ledger rows and the
  success criteria from gates ① through ④ (`references/clarity-ledger.md`).
- Wireframe fit: the design generalizes the validated concrete case (the ④
  contract) instead of silently changing structure.
- Architecture fit: for brownfield structural changes, the before/after
  architecture sketch names preserved, replaced, extended, and moved
  responsibilities, and the before model was checked against the real system
  where cheap.
- Precedent fit: the design follows the existing local grammar where possible
  and names any new precedent it introduces. If future work copies this artifact,
  the copied naming, structure, placement, order, tone, taxonomy, workflow, proof
  shape, policy language, component boundary, file location, or interface shape
  should not confuse the system.
- Evidence fit: claims about existing materials, audience, facts, examples, or
  system behavior are checked where cheap.
- Risk mitigation: security, migration, compatibility, performance,
  reputational, operational, and review risks have concrete mitigations or
  explicit accepted residual risk.
- Verification path: ⑦ Tasks can produce tasks with acceptance checks that
  would prove the design works.

## Output Shape

```text
Verdict: APPROVE | ITERATE | REJECT

Reason:
-

Required revisions:
-

Residual risks:
-
```

For `ITERATE` or `REJECT`, name the smallest change that would let the design
return to review. Do not expand into implementation planning; that belongs in
⑦ Tasks after the design is settled.

## Where the inputs come from

The critic pass reads, but does not produce, these artifacts:

- ① Intent — the locked necessity / why the design must answer.
- ③ Criteria — the purpose and acceptance checks the design must satisfy.
- ④ Wireframe — the locked contract and the validated concrete case.
- ⑤ Design — the generator and, for non-obvious choices, the
  RALPLAN-DR rationale (`references/decision-rationale.md`). The Principles
  and drivers, Fair alternatives, and Steelman antithesis review checks above
  correspond one-to-one to the RALPLAN-DR fields.

A design that lacks RALPLAN-DR for a non-obvious choice is usually an
automatic `ITERATE` — the critic cannot assess what was not recorded.
