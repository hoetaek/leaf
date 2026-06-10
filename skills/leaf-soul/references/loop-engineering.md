# Loop Engineering

This reference records the loop-engineering idea being adopted into
`leaf-soul`. Treat it as source material for conduct, not as a separate workflow
contract.

## Source

- Addy Osmani, "Loop Engineering", 2026-06-08.
  https://addyo.substack.com/p/loop-engineering

## Adopted Interpretation

Loop engineering moves the leverage point from prompting an agent turn by turn
to designing a system that finds work, hands it out, checks it, records state,
and decides what should happen next. The identity LEAF adopts is not "the person
who presses go." It is the loop steward: the person who designs the loop,
maintains its memory, keeps maker and checker separate where risk warrants it,
and remains accountable for verification and understanding.

## Building Blocks

Use these as a lens when LEAF work becomes recurring, parallel, or autonomous:

1. Automations
   - The heartbeat. They discover, triage, summarize, or retry work on a cadence.
   - Risk: a recurring bad instruction compounds.

2. Worktrees
   - Isolation for parallel agents so file edits do not collide.
   - Risk: isolation removes mechanical conflict but not review overload.

3. Skills
   - Durable project knowledge outside one conversation.
   - Risk: weak skill descriptions or stale instructions make the loop repeat
     bad assumptions.

4. Plugins and connectors
   - Access to real tools such as issue trackers, databases, staging APIs, and
     communication channels.
   - Risk: the loop can act in more places than it understands.

5. Sub-agents
   - Separate roles such as explorer, implementer, and verifier.
   - Risk: extra agents cost tokens and still need a trustworthy stop condition.

6. Memory
   - The spine of the loop: a repo file, board, issue, or other durable state
     outside a single model context.
   - Risk: without state, every cycle re-derives history and silently forgets
     what was tried.

## Conduct To Adopt

- Design loops so the next run can see what happened before.
- Put recurring context into skills instead of repeatedly pasting prompts.
- Separate maker and checker when the loop may run unattended or when the cost
  of a false "done" is high.
- Use worktrees or equivalent isolation before running parallel agents against
  the same repo.
- Keep a durable state spine that records what was found, tried, passed, failed,
  and left for the user.
- Treat loop output as a claim until verified. "Done" is not proof.
- Watch comprehension debt: the faster a loop ships work you did not write, the
  more deliberately you must inspect enough of it to stay the engineer.

## Adoption Boundary

Adopt:

- loop stewardship as part of LEAF's conduct identity;
- persistent state, explicit stop conditions, and maker/checker separation;
- skepticism about unattended loops and token/review costs;
- the principle "build the loop, stay the engineer."

Do not adopt:

- tool-specific commands as permanent LEAF requirements;
- automation for its own sake;
- a loop that exists to avoid understanding the work;
- a false sense that verification can be delegated away.
