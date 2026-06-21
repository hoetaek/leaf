# The Experiment Log

The core of an experiment: **run an independent, cheap probe to turn one guess
into a fact you can't doubt — then accumulate those facts as the ground the next
move stands on.** It is *leaf before tree* at the scale of a single fact:
validate one cheap, inspectable thing before building on it. **② Unknowns is this
machine's entry point; ④ Wireframe reuses it aimed at the answer.** SKILL.md and
`gates.md` name the experiment in passing and defer here for the mechanics.

What this is *not*: iterating an answer toward a metric — tweaking and measuring
over and over, keeping or discarding each change. That is tuning, a neighbor that
also runs-and-measures but aims to *improve* an answer, not *establish* a fact.
See "The neighbor" below.

## The core — three properties and an accumulation

- **Independent.** Verify *outside* the thing in doubt. Re-running the project's
  own code proves nothing when the code is what's in question — cross-check with
  curl, the browser, a standalone script, a separate reader. The methodic-doubt
  rule: build the next judgment only on a basis you couldn't have faked.
- **Cheap — and cheap is relative.** Weigh the cost of verifying *one element
  independently* against the cost of *approaching the whole*. A single probe is
  cheap not in the absolute but because the alternative — building the whole
  artifact to find out — is dear. This is exactly *leaf before tree*.
- **Indubitable.** A result is a fact only when its basis can't be reasonably
  doubted: a source, an observed output, an independent check — never an
  assertion. A *refuted* guess is also a fact ("X is not the cause") and earns
  its place on the ladder.
- **Accumulated — a ladder, not a list.** Facts chain: each established fact is
  the foundation the next experiment stands on, and the order is load-bearing.
  Record what each fact *stands on* and what it now *enables*, so the foundation
  is visible and a later doubt can be traced to the rung it undermines.

## Two probes, one core

The probe points one of two ways; both are cheap independent fact-probes, and
both feed the same ladder.

- **World probe (② Unknowns) — "is this true?"** Verify a fact, convention,
  source, or audience condition *before* any answer exists. The cheap element is
  one unknown; the dear whole is everything built on a wrong assumption.
- **Answer probe (④ Wireframe) — "is this answer right?"** One concrete
  instance — the cheap element — walked through by a real reader/operator
  establishes an indubitable fact about the answer ("this instance fails to
  convey X") before the whole artifact — the dear whole — is built. `gates.md` ②
  forbids using a ② probe to validate an answer shape; that is ④'s probe.

## The neighbor — tuning is not experiment

When you start tweaking an answer repeatedly and measuring against a metric —
keep this change, discard that one, recover when three in a row fail — you are
*tuning*, not gathering facts. It shares the run-and-measure surface but aims to
*improve* an answer, not *establish* a fact, so it does not live here. Drive it
from `03-Architect/08-execution.md` as execution work, or use the standalone
`/experiment` skill's debugging loop. Keep this file about facts.

## The fact / guess boundary

The spine. Never mix the two: a guess treated as fact drags the work the wrong
way for a long time.

| Claim | Type | Basis |
|---|---|---|
| … | fact | file:line, command output, independent check |
| … | guess | needs an independent cheap test |

## Where it lives

Keep the probing process in a sidecar; let only the established fact flow back to
the gate's index.

- **② Unknowns** — process in `01-Learn/02-experiments/{name}.md`; the verified
  fact goes back to `02-unknowns.md`. Third sibling of `02-references/`: same
  `02` prefix, same job of keeping `02-unknowns.md` a clean index of what is now
  known, not a scratchpad of every probe.
- **④ Wireframe** — in the wireframe file/folder.

Run outputs — logs, input/output samples, judge results — go in an
`{experiment-name}/` subfolder beside the log.

## The log — a fact ladder

One problem, one running log; append, don't restart. One guess per rung, so the
fact is attributable to its test. Record facts in dependency order:

```text
# [Problem / question]

## Guesses              — what we treat as true without proof (the table above)

## Fact ladder          — established facts, in the order they had to be earned
### Fact 1
- Guess tested   — the one hypothesis put at risk this rung
- Cheap independent test — what was run outside the thing in doubt; why it's
                   cheap (the dear whole it stands in for)
- Result         — the indubitable fact + a basis that can't be faked
                   (a refuted guess is a fact too)
- Stands on      — earlier fact(s) this needed ("—" if none)
- Enables        — the next experiment or judgment this now grounds

## Open guesses         — not yet converted; carried to ③ as explicit assumptions
```

## Technique repertoire — ways to make a probe independent and cheap

Pick by the question's nature. Each establishes a fact; none requires building
the whole.

1. **Baseline comparison** — run before- and after-versions on the same input;
   the fact is which is better on a named criterion, not a feeling.
2. **Red-team cases** — deliberately provoke the failure pattern; include normal
   cases so a fix isn't a new break. Define PASS/FAIL per case *before* running.
3. **LLM-as-judge** — a separate model scores against *observable* criteria (not
   "is it natural?" but "no `~고 했다` indirect-quote endings"). The separateness
   is what makes it independent.
4. **Assertions** — define what counts as the fact before running; `grep`-able
   checks become reusable across reruns.
5. **Blind comparison** — hide which output is old/new; ask only "which better
   meets the criterion," swap order, re-run. Strips the "newer must be better"
   bias. (Judge = absolute pass/fail; blind = relative which-wins.)
