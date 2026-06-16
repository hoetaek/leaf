# ① Intent

**What ① locks: you can state why this work is needed and, derived from that,
what is actually wanted.** The why is the problem definition; the what is the
artifact or result LEAF will help produce. ② covers everything that must be
learned to reach it well.

The what can be a concept, taxonomy, model, decision record, policy, plan,
document, UI, code change, or other artifact. Do not assume "real output" means
code or prose. If a concept is the thing that resolves the why, the concept is a
valid leaf output. If that concept later needs implementation, split the
implementation into a separate leaf instead of silently expanding the original
leaf.

Preserve what the user initially said, then refine the underlying intent. The
user may not know the purpose, success criteria, or output form yet; intent can
start as a hunch, curiosity, taste direction, discomfort, or "something like
this."

**Ask why before locking what.** The why is the question furthest from any
solution, so it carries the least solution-contamination, while the surface
request is often a means already dressed as an end. Stating the why derives or
corrects the what; when the derived intent differs from the surface request, that
gap is ①'s first discovery. Record both.

**Follow the why to where it actually lands.** Not every answer is a problem
definition. An external trigger is a real why but may define no problem: it fixes
the arbiter and caps the investment, and the problem-defining why sits one level
deeper. Curiosity or taste is a legitimate endpoint: lock the intent as
exploratory and stop digging. A felt sense marks a problem that exists but cannot
be articulated yet: keep the user's exact words, state the what negatively, and
hand the articulation to ② as an unknown. When no problem, obligation, curiosity,
or felt sense holds up, kill the work here.

**Competing whys derive competing whats.** Several motives at once may point at
different deliverables. A why that embeds a diagnosis splits into the problem
and the hypothesis; the hypothesis must stay out of the what until ② verifies it.

**Earn the answer.** A user who feels interrogated, or sees no stake in the
question, answers perfunctorily. Give each question its reason: the fork the
answer decides, a quick example, or what the previous answer already changed.

## Why / What Separation

Do not turn the user's raw wording directly into the locked intent. Separate
`why` from `what`.

- `Raw wording`: preserve the user's exact surface request, title, example, or
  discomfort.
- `Why`: name the underlying problem, cost, delay, risk, obligation, or missing
  capability that makes the work matter. Make it sharp; if the why is fuzzy,
  every later artifact can look plausible while solving the wrong problem.
- `Provisional what`: state the currently visible artifact or result LEAF might
  produce, but mark it as changeable during Learn. It may be a concept/model as
  much as a code change, document, UI, or other external artifact.
- `Locked intent`: write one sentence that preserves the why without prematurely
  freezing an unverified implementation direction.

If the why is stable but the what could change, keep the what provisional. Why
has priority: as long as the problem definition still holds, LEAF may switch the
what to a better artifact. Record the fact or premise that caused the switch. If
the new what would create a separate lifecycle, reviewer, success check, or code
surface, split it into another leaf.

Prefer locked intents that say what wrong state must be corrected or what
distinction must be learned, rather than intents that already prescribe the
implementation.

Ask or infer:

- What was the original request or impulse?
- Why is this needed: what problem, obligation, curiosity, or discomfort sits
  underneath?
- Is it a topic, a problem, a claim, a deliverable, or a deadline?
- What wording should not be lost?
- What is the provisional what, the currently visible artifact or result that
  LEAF might produce: concept, taxonomy, model, decision, document, code change,
  UI, migration, or something else?
- What question must ② answer before the right what can be chosen?
- What locked intent preserves the why without freezing an unverified
  implementation direction?

Gate to continue:

- Raw wording is captured.
- The why has been asked or inferred and followed to where it lands: a problem
  definition, an external obligation with the deeper why asked, curiosity locked
  as exploratory, or a felt sense deferred to ②, or the work was killed here.
- The current one-sentence intent is stated separately from the raw wording;
  where the derived intent differs from the surface request, the gap is recorded.
- Why and what are visibly separated: `Raw wording`, `Why`, `Provisional what`,
  and `Locked intent` are recorded as distinct fields or clearly distinct
  paragraphs.
- The locked intent does not merely restate the user's implementation-shaped
  wording. If the why is stable but the what could change, the locked intent
  names the wrong state to correct or the distinction to learn, and keeps the
  solution direction provisional.
- The provisional what names the intended artifact/result clearly enough to learn
  what it needs. Concept/model outputs are allowed; code changes are not silently
  bundled into a concept leaf.
- At least one ② question names what must be learned to choose the right what.
- Separate ideas are split or explicitly grouped.
- The core noun is named and stable. If it drifts across answers, halt and ask
  which noun is the actual work item before continuing.
- When the request has multiple possible outcomes, the top-level topology is
  named: outcomes, surfaces, integrations, or deliverables that can succeed or
  fail independently, including anything explicitly deferred.
- The work is still allowed to change shape or die.
- It is clear whether the user already has a purpose or success criterion, or
  needs context exploration to shape one.

The why locked here is itself a hypothesis, and every later gate can falsify it.
Return here when ② learning articulates a deferred felt sense or overturns the
stated need, when ④'s concrete instance shows the problem was misstated, or when
⑧ drafting / ⑨ review reveals the work is answering a different question. Re-lock
the intent explicitly, then resume only the gates that depended on what changed.
