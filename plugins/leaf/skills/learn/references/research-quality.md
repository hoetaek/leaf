# Research Quality

Read this reference whenever external facts can change a Learn judgment: broad
web research, disputed claims, recent facts, statistics, legal/policy/medical/
financial claims, citation-heavy synthesis, or a public source that normal
search/fetch/browser access cannot read.

This file deliberately carries only the reusable method. It does not vendor the
`insane-research` or `insane-search` engines.

## Minimal Rule

LEAF owns the learning structure and final synthesis.

- Use `insane-research` / `insane-research-codex`, when exposed in the current
  runtime, as a reference-producing scout for broad or citation-heavy research.
- Use `insane-search` / `insane-search-codex`, when exposed in the current
  runtime, only as a fallback for blocked public sources.
- If those capabilities are unavailable, continue with built-in search, fetch,
  browser, and explicit limitation notes.

Do not make either plugin a hard dependency of Learn.

## Source Rating

When external facts can change the user's judgment, rate the decisive sources by
default. Adjust depth, not the habit: a small Learn pass may only need ratings
for 2-3 load-bearing sources, while high-stakes or disputed claims need the full
ledger.

| Rating | Use for | Evidence strength |
|---|---|---|
| A | Primary or definitive sources: official records, standards, peer-reviewed systematic reviews, court/regulator documents, original datasets | Can support decisive claims |
| B | Authoritative sources: official docs, peer-reviewed original research, professional guidelines, institutional research | Strong evidence; verify if critical |
| C | Expert or reputable secondary sources: named experts, conference talks, case studies, reputable analysis | Good context; verify key claims |
| D | Leads and supporting material: preprints, expert blogs, press releases, trade publications | Use as leads, not proof |
| E | Weak material: anonymous claims, social posts, SEO farms, unsupported opinions, unverifiable AI output | Do not cite as evidence |

For fast-moving technical topics, current primary artifacts such as official
docs, changelogs, specs, repositories, and benchmark papers may outrank older
secondary commentary.

## Claim Rules

Use this ledger shape in scout reference files when research quality matters:

```md
| Claim | Sources | Rating | Status | Notes |
|---|---|---|---|---|
| ... | ... | A/B/C/D/E | verified / unresolved / refuted | primary source? conflict? |
```

Apply these thresholds:

- Definitive claims need A/B evidence, or a clear note that they remain
  `unresolved`.
- Numbers, market sizes, legal requirements, medical claims, security claims,
  and causal claims need a primary source or two independent strong sources.
- If sources conflict, do not smooth it over; record the conflict and mark the
  claim `unresolved` until the deciding source is found.
- Treat sidecar sources such as archives, metadata, snippets, or cached copies as
  provenance-tagged support unless they preserve the original source clearly.

## Citation Minimum

Every factual claim that matters downstream needs enough citation detail for the
next agent to verify it without redoing the search:

- author or organization;
- publication or retrieval date;
- title;
- direct URL or DOI;
- section/page/line when the source is long;
- source rating when quality matters.

Avoid vague phrases such as "studies show", "experts say", "reports indicate",
or "recent research" unless the named source is attached.

## Access Fallback

Use blocked-source access only for public content. Do not try to bypass login
walls, paywalls, private content, or access controls.

When normal access fails, record:

```md
- access path: built-in search / fetch / browser / insane-research / insane-search / unavailable
- blocked signal: status code, WAF marker, empty page, login/paywall marker, or platform limit
- boundary: public content only; auth/paywall/private content not attempted
```

If an optional access capability succeeds, cite the original source and note the
access path. If it fails, keep the failure trace short and move on; do not let
access recovery replace the Learn question.

## Attribution

This method file adapts compact research-quality and public-source access
patterns from fivetaku's MIT-licensed `insane-research` and `insane-search`
projects. See `plugins/leaf/NOTICE.md`.
