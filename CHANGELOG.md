# Changelog

## [0.3.1] — 2026-08-08

### Fixed
- **CI failed on both jobs that build, before a single test ran.** Every job
  passes `--locked`, and `Cargo.lock` was in `.gitignore` — so a fresh checkout
  had no lockfile and cargo refused with *"the lock file needs to be updated but
  --locked was passed"*. Two red jobs, one cause, and nothing to do with the
  code they were checking.

  The lockfile is now committed. The usual advice against committing one for a
  library is about leaving a *consumer's* resolver free; it says nothing about
  CI, where a missing lockfile is not freedom but a hard failure. This crate has
  zero dependencies, so the file contains itself and constrains nobody.

  It also buys something. `--locked` now catches a manifest bumped without its
  lockfile — the exact defect two other repositories in this organisation
  shipped this month, each time surfacing as a confusing message about a
  lockfile rather than about the version that caused it.

- A range loop indexing two slices in `explain`, rewritten to iterate the pair.
  The local clippy is older than the one CI uses and did not flag it. This
  project has paid for that gap three times by discovering the lint in a red
  build instead of before pushing, so the pattern is now removed on sight
  rather than when it is reported.

## [0.3.0] — 2026-08-08

### Added
- **`analysis`: twelve functions over a score.** The combiner answers what a
  project scores; these answer the questions that follow, and every one existed
  as hand-written arithmetic somewhere first.

  `RULE_VERSION` and `to_wire`/`wire_rule_version` give a published score its
  provenance, so two snapshots produced by different rules can be refused
  instead of silently compared. `gap_to_gate` takes the gate out of the call
  site. `cheapest_lift` says what would clear it, and returns `None` rather than
  a number that would not work. `explain` renders a waterfall. `dominance_pct`
  says whether one rule is carrying a score. `resolution` and `clumping` are the
  diagnostics that should have existed from the start — the defect where the
  rule returned nine distinct values across a hundred projects survived thirteen
  releases because nothing measured it. `rank_key` fixes a total order.
  `verify_corpus` checks the linked build rather than the built one. `simulate`
  scores a hypothetical repository.

### Fixed while writing the above
- **`explain` was designed around arithmetic that does not hold.** By-removal
  contributions do not sum to the total: under a saturating combiner the
  remaining evidence partly covers for anything removed, so each removal costs
  more than its share. The first draft summed them into a running column and
  produced 43 for a score of 82.

  Removal now decides only the *order*; the column is built from increments
  along that order and ends on the published score. A test pins the non-summing
  property so the obvious mistake is caught rather than repeated.

## [0.2.0] — 2026-08-04

### Added
- **Continuous integration, which this repository did not have.** Four jobs, and
  the one that matters is `cross_target_identical`: the corpus is emitted
  natively and from WebAssembly and the two outputs are diffed, so a single
  differing byte fails the build. The README has claimed since the first release
  that the same score must come out of a Linux runner and out of the same code
  in a browser, and that *identical* is a stronger word than *close*. Nothing
  verified it. Now something does.

  Adding CI immediately found three defects clippy had been flagging with nobody
  to see them, which is the argument for having it rather than a footnote to it.

- **`corpus` is a public module.** The conformance vectors lived in `tests/`, so
  anything outside the test harness that needed them had to copy them — and a
  copy was made and was wrong within the hour, transcribing one project's
  evidence as three items instead of four and producing 90 where the pinned
  value is 93. Hand-transcribed vectors diverge; that is a property of copying
  rather than a lapse to be more careful about. There is now one list, and a
  third party implementing this rule in another language can read it from the
  crate instead of from a document.

- A section documenting how to **recompute any published score** from public
  data. The keyword table lives in the scanner, but its output does not: each
  project's evidence vector is published beside its score and refreshed every
  three hours, which is what makes a score disputable rather than merely
  visible.

### Changed
- Three `if`/`else if` chains over a signed comparison are `match` on
  `Ordering`, which states the third case — evidence worth zero points is
  neither support nor penalty — instead of leaving it implied.

All notable changes to axonos-brs are recorded here. Format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/); versions follow
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.1] — 2026-08-01

### Added
- Licence texts (`LICENSE-APACHE`, `LICENSE-MIT`), the `NOTICE` that
  Apache-2.0 section 4(d) obliges a redistributor to retain, `CITATION.cff`
  with the author listed first, and this changelog.

  The crate declared `Apache-2.0 OR MIT` in its manifest and its README from
  the first release and shipped neither text. A dual-licence declaration
  without the licences does not grant what it announces: a reader who wants to
  depend on this had nothing to read, and Apache-2.0's attribution clause
  cannot be honoured against a `NOTICE` that does not exist. That is a defect
  in what the repository *is*, not in what it does, which is why it is recorded
  here rather than quietly added.

  No code changed. The version moves because the artefact a consumer receives
  is materially different: it can now be depended on under the terms it always
  claimed.

---

<sub>**axonos-brs v0.1.1** · © 2026 Denis Yermakou · Apache-2.0 OR MIT ·
authored for [The AxonOS Project](https://axonos.org) · connect@axonos.org</sub>
