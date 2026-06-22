# LEAF Autopilot Approval Policy

Use this reference when deciding whether `leaf-autopilot` may continue without
asking the user.

## Decision Model

`leaf-autopilot` uses three layers:

1. **Human-reviewed triple** — `why`, `what`, and `wireframe` are reviewed before
   autopilot starts. This is the ownership boundary.
2. **Automatic gate review** — after the triple, the agent may draft, grill,
   revise, test, and review LEAF gates without asking at every gate.
3. **Hard stops** — when risk or ownership exceeds the pre-authorized lane, stop
   and ask.

## May Continue Automatically

Continue without asking when all are true:

- the action follows the locked triple and current criteria;
- the change is local to the repo or `.leaf/` workspace;
- the action is reversible or reviewable;
- no credential, production, public sharing, cost, legal, security, privacy, or
  permission boundary is crossed;
- the required evidence can be gathered locally;
- the next gate can consume the previous gate without inventing a new why, what,
  wireframe, core noun, or scope.

Examples:

- writing or revising LEAF gate files;
- running `leaf doctor`, tests, lint, build, or local validation commands;
- creating the promised skill files in the repo;
- running a local dry run or forward-test that does not publish or deploy;
- using subagents for critic/review when their write scopes are bounded.

## Must Stop

Stop when any of these are true:

- the triple is absent, provisional, stale, or contradicted;
- a later gate wants a different deliverable or wireframe form;
- destructive or hard-to-revert operations are needed;
- secrets, credentials, external accounts, production systems, deployment, or
  paid services are involved;
- public/external communication or sharing is involved;
- security, privacy, legal, policy, or permission boundaries are affected;
- the task requires user taste, organizational risk tolerance, or stakeholder
  judgment not encoded in the triple;
- the same failure repeats three times;
- validation fails and the next fix would widen scope.

## Recording Delegation

When continuing automatically at a former approval point, record:

- the pre-authorized basis: the locked triple and this policy;
- what was reviewed automatically;
- what evidence passed;
- what hard stops were checked;
- what remains unresolved.

Use the nearest durable surface:

- `00-status.md` for current phase, gate, next action, return, or stop;
- the current gate file for gate-specific review evidence;
- `03-Architect/08-execution.md` for execution sessions;
- `04-Feedback/09-review.md` and `10-retrospect.md` for close-out findings.

## Return Rule

If an automatic gate finds that the locked triple is wrong, return to Learn. Do
not patch around it downstream. Record what falsified the triple, reopen only
the affected fields, and wait for human review before continuing.
