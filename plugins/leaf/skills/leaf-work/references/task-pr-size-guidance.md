# Task and PR Size Guidance

Use this when slicing the ⑦ task graph into reviewable units. The numbers are
heuristics for reviewability — tripwires that mean "justify or split", not
schema rules.

## Primary Rule

One task or PR should make one self-contained, reviewable change with its own
acceptance checks. Prefer small slices, but do not split so far that the change
stops making sense in isolation.

## Size Heuristic

- `small`: 1–5 meaningful files, <= 200 meaningful changed lines.
- `medium`: 6–10 meaningful files, <= 400 meaningful changed lines.
- `large-justified`: larger than that. Split first; keep as one only with an
  explicit coupling reason and a review/check plan.
- `usually too large`: 20+ meaningful files for ordinary feature/refactor
  work, or around 50 files for non-mechanical review work.

Count generated files, lockfiles, snapshots, mass deletes, and mechanical
renames separately from meaningful review files. A broad mechanical change can
be reviewable when the reviewer is validating the transformation, not reasoning
through each changed site independently.

## Split Triggers

Split or justify when a slice:

- mixes multiple concepts, such as model/schema, interface contract,
  persistence, runtime behavior, docs, and tests;
- touches more than one subsystem without a single coupling reason;
- needs multiple reviewers with unrelated ownership areas;
- lacks a clear review order through the changed files;
- would take more than about 60 minutes for a careful reviewer to inspect;
- has no acceptance check beyond "looks good".

## Acceptable Large Cases

Keep a larger slice together when splitting would be worse:

- one atomic migration boundary;
- trusted mechanical refactor or rename;
- generated output required by one source change;
- deletion-heavy cleanup;
- API plus one representative usage and tests where separating them would hide
  the API's implications;
- compatibility shims that must land with the behavior they preserve.

For these, label the slice `large-justified` and include:

- why the files move together;
- which files or modules are the review starting points;
- which files are generated or mechanical;
- focused checks that reduce review risk.

## Sources

- GitHub Docs, "Best practices for pull requests": recommends small, focused
  PRs with a single purpose, and review-order guidance when multiple files
  change.
  https://docs.github.com/en/pull-requests/collaborating-with-pull-requests/getting-started/best-practices-for-pull-requests
- Google Engineering Practices, "Small CLs": defines the right size as one
  self-contained change; notes that file spread affects size and that 200 lines
  across 50 files is usually too large.
  https://google.github.io/eng-practices/review/developer/small-cls.html
- SmartBear, "Best Practices for Code Review": recommends reviewing no more
  than 200-400 LOC at a time and limiting review sessions to roughly 60-90
  minutes.
  https://smartbear.com/learn/code-review/best-practices-for-peer-code-review/
- Carbon Language, "Code review": recommends changes as small as possible while
  remaining self-contained and addressing one thing.
  https://docs.carbon-lang.dev/docs/project/code_review.html
- BrainFANS, "Conducting a code review": treats a large number of changed files
  as a review difficulty and a signal that a PR may be doing too many things.
  https://ejh243.github.io/BrainFANS/Developer-information/Code-review/Conducting-a-code-review
- Dogan and Tuzun, "Enhanced code reviews using pull request based change
  impact analysis": models PR risk through changed files, churned files, buggy
  files, co-changing files, and changed lines.
  https://link.springer.com/article/10.1007/s10664-024-10600-2
