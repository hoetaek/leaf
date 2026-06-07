# The ③–⑤ Engine

The middle of LEAF is a single machine: Criteria, Wireframe, and Design are not
independent steps but a **criteria → instance → generator chain**. This file is
the full home of that machine. SKILL.md states it compressed; read here for the
mechanics, the falsification loop, and the merge rule. `gates.md` covers the
surrounding gates (①–②, ⑦–⑩) and defers to this file for ③–⑤.

This engine is a self-similar miniature of LEAF as a whole. The same shape —
*learn → become able to judge → generalize* — runs at three scales: across LEAF
(Learn → Example → Architect), inside this engine (③ → ④ → ⑤), and inside Learn
itself (② coming to know → being able to judge). At every scale the move is the
same: reach a point where you can judge, and only then go on. That is why ②→③ is
the same kind of produce/consume edge as ③→④ and ④→⑤ — only someone who can
judge can set the criterion.

## The graph

```text
                              LEAF — component graph
                              ══════════════════════

      LEARN                EXAMPLE                 ARCHITECT             FEEDBACK
  (find the want)     (lock the criterion)       (build from it)       (sync & learn)
  ┌────────────┐     ┌─────────────────┐        ┌──────────────┐      ┌────────────┐
─▶│ ① Intent   │ ─▶  │ ③ Criteria      │  ─▶    │ ⑤ Design     │ ─▶   │ ⑨ Review   │
  │ ② Unknowns │     │ ④ Wireframe     │        │ ⑦ Tasks      │      │ ⑩ Retro    │
  │ + Context  │     │                 │        │ ⑧ Artifact   │      │            │
  └────────────┘     └─────────────────┘        └──────────────┘      └────────────┘
        ▲                  │      │                    │                    │
        ╰──────────────────┴──────┴────────────────────┴────────────────────╯
            loop back to ② whenever an assumption is overturned


  ┌──────────────────────────── the ③–⑤ engine ───────────────────────────────┐
  │                                                                            │
  │  ③ CRITERIA = ARBITER + TEST ─────────────────────────────────────────┐   │
  │     │   purpose = intended change; highest authority in the engine    │   │
  │     │   requirements = observable checks authored before the answer   │   │
  │     ▼                                                                  │   │
  │  ④ WIREFRAME = INSTANCE ──┬──▶ CONTRACT ─ fixed, must not change       │   │
  │     (one rendered case)   └──▶ VARIATION POINT ─ what varies / axis / range │
  │     │                                                                      │
  │     │   can FALSIFY criteria; appeal to purpose inside ③                  │
  │     │                                                                      │
  │     │   ▼ ⑤ CONSUMES the contract (never rediscovers it)                   │
  │     ▼                                                                      │
  │  ⑤ DESIGN = GENERATOR ──▶ produces EVERY valid instance the contract       │
  │              allows; behavior across each variation point's FULL range     │
  │              (empty · overflow · edge · timing · failure)                  │
  └────────────────────────────────────────────────────────────────────────────┘

  legend
  ──▶   flow / produce → consume        authoring order & authority:
  ╌╌▶   falsification (answer ↯ test)     ① ▸ ③ ▸ ④ ▸ ⑤
  ✂ never merge across a produce/consume   (earlier = more authority,
    edge:  ③→④  and  ④→⑤  stay separate     less solution-contamination)
  appeals climb the same chain: instance ↯ test → ③ purpose arbitrates;
    purpose ↯ → the necessity (why) locked at ① arbitrates
```

## ③ Criteria — the arbiter + test

Define the change the work must cause and what must be true for a concrete
instance to pass. Criteria combines purpose and requirements because both are
authored before the answer exists and both serve judgment:

- **Purpose** is the intended change. Within the ③–⑤ engine it is authored
  furthest from the solution, so it has the highest authority here; when
  purpose itself is disputed, the appeal climbs one level up — to the
  necessity (the why) locked at ① Intent.
- **Requirements** are the observable conditions and constraints: audience,
  scope, claims to support, evidence sources, tone, length, format, deadline,
  non-goals, quality bars, and tradeoff principles.

Criteria is design-independent. It must be specific enough to reject a bad
wireframe, but it is still only a proxy for purpose — so the instance can
falsify a requirement.

## ④ Wireframe — the instance + contract

Validate one concrete case before generalizing. A wireframe is built from
placeholders, and **every placeholder hides an undeclared contract**. Mock data
renders one instance; it does not declare the schema behind it. So ④ yields two
paired outputs:

- the **instance** — one concrete case rendered with realistic mock data;
- the **contract** — what every placeholder commits to and must not change.

What the instance leaves free is a **variation point**: a positively specified
axis of variation — *what varies, along what axis, within what range* — not a
leftover and not "whatever the executor later decides."

Two passes:

1. **Text-first wireframe (required by default)** — ASCII layout, command
   transcript, sequence sketch, table/state matrix, or outline with placeholder
   evidence. Validates structure while it is cheap to throw away.
2. **Artifact-specific wireframe (when the medium is needed)** — HTML for web,
   generated TOML/YAML for config, request/response examples for APIs, a
   one-page skeleton for proposals. Only after the text-first pass works. For
   visual outputs this may include the concrete visual treatment the user
   approves — still a case to validate, not the reusable system.

**Locking the contract** (do this before leaving ④):

1. List every placeholder/mock in the instance.
2. Declare what can be declared — schema, types, enums, tempo, aspect ratio
   (declarative contract).
3. For inherently ostensive contracts — vibe, look, tone — lock the validated
   instance itself as the reference ("match this"). The contract may be
   emergent, so ④ can loop: generate several instances to find the one worth
   locking.
4. Swap each placeholder for the real asset's actual constraints (real schema,
   real evidence, real audio length) and confirm the instance still holds.
5. Only then leave. The contract is declared here; ⑤ consumes it and must never
   rediscover it.

## ⑤ Design — the generator

Build the generator, not another instance. ④ locked one validated instance plus
its contract; Design generalizes it into the reusable rules that produce *every*
valid instance the contract allows. The contract is an **input** consumed from
④, never rediscovered here.

The central work is **behavior across the full range of each variation point** —
the empty, overflow, and edge cases, responsive breakpoints, and state ranges a
single mock instance could not reveal. Treat failure and timing as variation
axes too: a happy-path instance never shows them. Set coverage depth by
likelihood × impact (robust handling for high/high; graceful failure for
rare/low).

For each variation point declared in ④, walk its full range — do not brainstorm
"all edge cases" untethered:

- **Input domain** — empty, too long, malformed, special characters, min/max.
- **Boundaries** — edges of each range (1–100 field → test 0, 1, 100, 101);
  pagination edges, timeout thresholds, rate limits.
- **Error states** — network failure, permission denied, not found, concurrent
  modification, session expiry. Document scenario and expected behavior.
- **Concurrency / timing** — two users at once, double-click, data changed
  between load and save (TOCTOU).
- **Recovery path** — for each error: what message, can they retry, is data
  preserved.

A new axis found here — a failure mode or timing case the instance never showed
— loops back to ④ to declare its contract before its handling is designed.

## The two rules that hold the chain together

**Falsification + arbiter.** Criteria is only a proxy for purpose. Refining the
instance (④) can reveal a criterion was wrong — the answer falsifies the test.
When the instance and a criterion conflict, the arbiter is the purpose inside ③.
Fix whichever fails that purpose, then resume. Falsification does not stop at ④:
the ⑤ generator is an inductive leap from one instance, so it is itself falsified
at ⑥ Critic before ⑦ Tasks builds on it.

**Never merge across a produce/consume edge.**

- **③→④** must stay separate — the criteria vs the concrete answer that must
  pass them. Because the answer can falsify the criteria, their disagreement
  must stay visible.
- **④→⑤** must stay separate — the locked contract vs the generator that
  consumes it.

So `03+04`, `04+05`, and `03+04+05` are forbidden. Keep ③, ④, and ⑤ as separate
files/folders even for tiny work.

## Routing a mid-work discovery

Ask: *would this still be true with a totally different example satisfying the
same intent?*

- **Yes** → it is a criterion (③).
- **No** → it is this instance's contract (④).
