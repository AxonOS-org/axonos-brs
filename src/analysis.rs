// SPDX-License-Identifier: Apache-2.0 OR MIT
// SPDX-FileCopyrightText: 2026 Denis Yermakou <connect@axonos.org>
//
// Part of axonos-brs. Dual-licensed Apache-2.0 OR MIT at your option.
// Authored by Denis Yermakou for The AxonOS Project — https://axonos.org

//! Analysis over a score: provenance, explanation, diagnosis, ordering.
//!
//! The combiner in [`crate`] answers one question — what is this project's
//! score. Everything here answers the questions that follow, and each was
//! written because somebody had to answer it by hand first:
//!
//! - *Which rule produced this number?* A published score outlives the release
//!   that computed it, and without a rule identifier a reader comparing two
//!   snapshots cannot tell a project that improved from a rule that changed.
//! - *How far short is it?* The map computes this by subtraction at the call
//!   site, which means the gate lives in two places.
//! - *What would move it?* A maintainer looking at a low score wants one
//!   actionable sentence, not a combinator.
//! - *Is this score ranking anything?* A scoring rule that returns nine
//!   distinct values across a hundred projects is a category label wearing a
//!   number. That defect shipped here once and went unnoticed for thirteen
//!   releases because nothing measured it.
//!
//! All integer, all `no_std`, no allocation. Where a function needs somewhere
//! to put results, the caller supplies the slice.

use crate::{attribute, score, score_with, Contribution, Evidence, Kind, Score, GATE, ONE};

/// Identifier of the scoring rule that produced a score.
///
/// **1.** Bumped whenever the arithmetic changes in a way that can move a
/// published number, which is not the same as the crate version: a release that
/// only adds analysis leaves this alone.
///
/// A score published without it is a number whose meaning depends on when it
/// was computed, and comparing two snapshots across a rule change silently
/// compares different things. Carrying the identifier makes that comparison
/// refusable instead of wrong.
pub const RULE_VERSION: u16 = 2;

/// How far a score sits from the gate.
///
/// **2.** Positive when short, negative when clear, zero exactly at the gate.
/// The map computed this by subtraction at the call site, which put [`GATE`] in
/// two places and invited them to drift.
pub const fn gap_to_gate(brs: u8) -> i16 {
    GATE as i16 - brs as i16
}

/// What a project would need to add to clear the gate.
///
/// **3.** Returns the smallest evidence weight, in points, that lifts the score
/// to the gate — or `None` if it already clears, and `Some(0)` never happens.
/// `None` for an unreachable gate too: a project deep in negative evidence
/// cannot be lifted by one positive signal, and saying so is more use than a
/// number that would not work.
///
/// This is the honest form of "what would raise this". It does not tell a
/// maintainer to add keywords; it says how much verifiable evidence is missing,
/// and the caller turns that into a sentence.
pub fn cheapest_lift(evidence: &[Evidence], kind: Kind) -> Option<i32> {
    if score(evidence).brs >= GATE {
        return None;
    }
    // Points are the currency the rules are written in, so search them
    // directly. The range covers every value the rule table can emit; beyond
    // it the weight is capped and adding more changes nothing.
    let mut points = 1;
    while points <= 100 {
        if score_with(evidence, Evidence::new(kind, points)) >= GATE {
            return Some(points);
        }
        points += 1;
    }
    None
}

/// One line of an explanation, ordered strongest first.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Step {
    /// Which evidence this line describes, as an index into the input.
    pub index: usize,
    /// The kind of evidence.
    pub kind: Kind,
    /// Points contributed to the published score, by removal.
    pub delta: i16,
    /// The running score after this line, so a reader can follow the arithmetic
    /// rather than trust the total.
    pub running: u8,
}

/// Build an explanation a reader can follow line by line.
///
/// **4.** A waterfall: ordered strongest first, each line carrying what that
/// evidence added *given everything above it*, and a running column that ends
/// on the published score.
///
/// The two-pass shape is forced by the arithmetic and is worth understanding
/// before changing it. By-removal contributions — what the score loses if this
/// one evidence disappears — are the right way to rank, because they measure
/// each signal against the whole. But **they do not sum to the total**. Under a
/// saturating combiner, removing A alone costs more than A's share, since the
/// remaining evidence partly covers for it; add those removals up and you
/// overshoot. The first draft of this function did exactly that and produced a
/// column ending at 43 for a score of 82.
///
/// So removal decides the *order*, and the column is built from *increments*:
/// the score of the first line, then the first two, and so on. Each delta is
/// what that evidence added in that position, the last running value is the
/// score itself, and a reader can check the arithmetic instead of trusting it.
///
/// Returns how many steps were written. `out` shorter than `evidence` truncates
/// after the strongest, which is the useful truncation — though a truncated
/// column no longer reaches the score, and a caller rendering one should say so.
pub fn explain(evidence: &[Evidence], out: &mut [Step]) -> usize {
    if evidence.is_empty() || out.is_empty() {
        return 0;
    }
    let mut contrib = [Contribution { index: 0, delta: 0 }; 32];
    let n = attribute(evidence, &mut contrib);
    let n = n.min(evidence.len()).min(32);

    // Selection sort by descending delta, stable on ties by original index.
    // Sorting in place without allocation, and the stability matters: two rules
    // contributing equally must not swap places between runs, or a rendered
    // explanation becomes non-deterministic for no reason.
    let mut taken = [false; 32];
    let mut order = [0usize; 32];
    let mut chosen = 0;
    while chosen < n {
        // Iterate the pair rather than an index into both. A range loop over
        // two slices is the shape a newer clippy flags, and this project has
        // paid for that lint three times by discovering it in CI instead of
        // here.
        let mut best: Option<usize> = None;
        for (i, (c, &t)) in contrib.iter().zip(taken.iter()).enumerate().take(n) {
            if t {
                continue;
            }
            match best {
                None => best = Some(i),
                Some(b) if c.delta > contrib[b].delta => best = Some(i),
                _ => {}
            }
        }
        let Some(b) = best else { break };
        taken[b] = true;
        order[chosen] = contrib[b].index;
        chosen += 1;
    }

    // Second pass: score the prefix at each step. This is what makes the
    // column walk to the total, and it is only correct because the order is
    // already fixed — an incremental contribution is a function of position.
    let mut prefix = [Evidence::new(Kind::Neuro, 0); 32];
    let mut running_prev: u8 = 0;
    let mut written = 0;
    while written < out.len() && written < chosen {
        let idx = order[written];
        prefix[written] = evidence[idx.min(evidence.len() - 1)];
        let running = score(&prefix[..=written]).brs;
        out[written] = Step {
            index: idx,
            // The kind is read back from the input rather than carried in the
            // contribution: `Contribution` is deliberately narrow, and widening
            // it to serve one caller would put the same fact in two places.
            kind: evidence[idx.min(evidence.len() - 1)].kind,
            delta: running as i16 - running_prev as i16,
            running,
        };
        running_prev = running;
        written += 1;
    }
    written
}

/// Whether one piece of evidence dominates the score.
///
/// **5.** Returns the largest single contribution as a percentage of the
/// published score. A rule responsible for most of a score is a rule the score
/// is really measuring, and a table where that happens often is a table with
/// one rule and some decoration.
pub fn dominance_pct(evidence: &[Evidence]) -> u8 {
    let total = score(evidence).brs;
    if total == 0 {
        return 0;
    }
    let mut contrib = [Contribution { index: 0, delta: 0 }; 32];
    let n = attribute(evidence, &mut contrib).min(32);
    let mut top: i16 = 0;
    for c in contrib.iter().take(n) {
        if c.delta > top {
            top = c.delta;
        }
    }
    ((top.max(0) as u32 * 100) / total as u32).min(100) as u8
}

/// How many distinct scores a set of projects produces.
///
/// **6.** The diagnostic that should have existed from the start. A rule
/// returning nine distinct values across a hundred projects is a category label
/// wearing a number, and that defect shipped here and survived thirteen
/// releases because nothing measured it.
///
/// `scores` is modified in place as scratch. Returns the count of distinct
/// values, which is the number worth watching over time.
pub fn resolution(scores: &mut [u8]) -> usize {
    if scores.is_empty() {
        return 0;
    }
    scores.sort_unstable();
    let mut distinct = 1;
    for i in 1..scores.len() {
        if scores[i] != scores[i - 1] {
            distinct += 1;
        }
    }
    distinct
}

/// How many of a set sit at the ceiling or on the gate.
///
/// **7.** The other half of the resolution diagnostic. Nineteen per cent at
/// exactly 100 and thirty-one per cent at exactly the gate was what made the
/// old rule useless, and neither shows up in a mean.
///
/// Returns `(at_ceiling, at_gate, at_floor)`.
pub fn clumping(scores: &[u8]) -> (usize, usize, usize) {
    let mut ceiling = 0;
    let mut gate = 0;
    let mut floor = 0;
    for &s in scores {
        if s >= 99 {
            ceiling += 1;
        }
        if s == GATE {
            gate += 1;
        }
        if s == 0 {
            floor += 1;
        }
    }
    (ceiling, gate, floor)
}

/// Compare two scored projects with a documented total order.
///
/// **8.** Descending by score, then ascending by the tie-breaker the caller
/// supplies — normally a name. Ranking is where non-determinism enters a
/// pipeline: a sort with an incomplete comparator produces a different order on
/// a different machine, and a map that reorders itself between runs looks
/// broken even when every number is right.
pub fn rank_key(brs: u8, tiebreak: u32) -> u64 {
    // Score descending in the high bits, tiebreak ascending in the low.
    (((99 - brs.min(99)) as u64) << 32) | tiebreak as u64
}

/// A score in eight bytes, for storage and for the wire.
///
/// **9.** Fixed layout, big-endian, no serialiser. A published score that
/// crosses a boundary should carry the rule that produced it, and a format that
/// is one function long is one a second implementation can reproduce from the
/// documentation.
///
/// Layout: rule version (2), score (1), decision (1), tier (1), reserved (1),
/// positive ppm truncated to 16 bits (2).
pub fn to_wire(s: &Score) -> [u8; 8] {
    let ppm16 = ((s.positive_ppm.min(ONE) * 65_535) / ONE) as u16;
    [
        (RULE_VERSION >> 8) as u8,
        RULE_VERSION as u8,
        s.brs,
        s.decision as u8,
        s.tier as u8,
        0,
        (ppm16 >> 8) as u8,
        ppm16 as u8,
    ]
}

/// Read the rule version out of an encoded score.
///
/// **10.** Enough on its own to refuse a comparison: two snapshots produced by
/// different rules are not comparable, and the cheapest way to enforce that is
/// to make the rule readable without decoding the rest.
pub const fn wire_rule_version(w: &[u8; 8]) -> u16 {
    ((w[0] as u16) << 8) | w[1] as u16
}

/// Check that the crate still reproduces its own pinned corpus.
///
/// **11.** A consumer can assert this at start-up. The vectors are already
/// checked in CI, but CI checks the version CI built; this checks the version
/// actually linked, which is the one that will produce the numbers.
///
/// Returns the index of the first vector that disagrees, or `None`.
pub fn verify_corpus() -> Option<usize> {
    crate::corpus::CORPUS
        .iter()
        .position(|v| score(v.evidence).brs != v.expect)
}

/// Score a hypothetical evidence set without disturbing the real one.
///
/// **12.** The counterfactual a maintainer actually wants: not "add one signal"
/// but "here is what my repository would look like with topics declared and a
/// standard named". Returns the new score and the movement.
pub fn simulate(current: &[Evidence], proposed: &[Evidence]) -> (u8, i16) {
    let before = score(current).brs;
    let after = score(proposed).brs;
    (after, after as i16 - before as i16)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn core(t: u32) -> Evidence {
        Evidence::with_terms(Kind::Core, 55, t)
    }
    fn modality(t: u32) -> Evidence {
        Evidence::with_terms(Kind::Modality, 40, t)
    }
    fn neuro() -> Evidence {
        Evidence::new(Kind::Neuro, 25)
    }
    fn penalty() -> Evidence {
        Evidence::new(Kind::Negative, -40)
    }

    #[test]
    fn the_gap_is_signed_and_zero_exactly_at_the_gate() {
        assert_eq!(gap_to_gate(GATE), 0);
        assert_eq!(gap_to_gate(GATE - 4), 4);
        assert_eq!(gap_to_gate(GATE + 10), -10);
    }

    #[test]
    fn a_project_that_clears_needs_no_lift() {
        assert_eq!(cheapest_lift(&[modality(1)], Kind::Modality), None);
    }

    #[test]
    fn the_cheapest_lift_actually_lifts() {
        let weak = [neuro()];
        let need = cheapest_lift(&weak, Kind::Modality).expect("a weak project can be lifted");
        assert!(need > 0);
        assert!(score_with(&weak, Evidence::new(Kind::Modality, need)) >= GATE);
        // And it is the *cheapest*: one point less must not clear.
        assert!(score_with(&weak, Evidence::new(Kind::Modality, need - 1)) < GATE);
    }

    #[test]
    fn an_unliftable_project_says_so_rather_than_guessing() {
        // Deep in penalty: no single positive signal within the rule table's
        // range clears the gate, and reporting a number that would not work is
        // worse than reporting none.
        let sunk = [neuro(), penalty(), penalty(), penalty()];
        assert_eq!(cheapest_lift(&sunk, Kind::Modality), None);
    }

    #[test]
    fn an_explanation_is_ordered_and_its_running_total_reaches_the_score() {
        let ev = [core(2), modality(1), neuro()];
        let mut steps = [Step {
            index: 0,
            kind: Kind::Neuro,
            delta: 0,
            running: 0,
        }; 4];
        let n = explain(&ev, &mut steps);
        assert_eq!(n, 3);
        for w in steps[..n].windows(2) {
            assert!(w[0].delta >= w[1].delta, "strongest first");
        }
        assert_eq!(
            steps[n - 1].running,
            score(&ev).brs,
            "the column must walk to the score"
        );
    }

    #[test]
    fn an_explanation_truncates_to_the_strongest() {
        let ev = [core(2), modality(1), neuro()];
        let mut steps = [Step {
            index: 0,
            kind: Kind::Neuro,
            delta: 0,
            running: 0,
        }; 1];
        assert_eq!(explain(&ev, &mut steps), 1);
        assert_eq!(
            steps[0].kind,
            Kind::Core,
            "the strongest survives truncation"
        );
    }

    #[test]
    fn one_signal_alone_dominates_completely() {
        assert_eq!(dominance_pct(&[core(1)]), 100);
    }

    #[test]
    fn a_well_evidenced_project_is_not_dominated_by_one_rule() {
        let ev = [core(2), modality(2), neuro()];
        let d = dominance_pct(&ev);
        assert!(d < 80, "no single rule should carry a rich project: {d}%");
    }

    #[test]
    fn resolution_counts_distinct_values_not_projects() {
        let mut s = [55, 55, 40, 40, 40, 92];
        assert_eq!(resolution(&mut s), 3);
        assert_eq!(resolution(&mut []), 0);
    }

    #[test]
    fn clumping_finds_the_shape_a_mean_would_hide() {
        let s = [99, 99, GATE, GATE, GATE, 0, 61];
        let (ceiling, gate, floor) = clumping(&s);
        assert_eq!((ceiling, gate, floor), (2, 3, 1));
    }

    #[test]
    fn ranking_is_a_total_order_with_score_first() {
        assert!(
            rank_key(92, 5) < rank_key(61, 1),
            "higher score ranks first"
        );
        assert!(
            rank_key(61, 1) < rank_key(61, 2),
            "ties break by the key given"
        );
        assert!(rank_key(99, u32::MAX) < rank_key(98, 0));
    }

    #[test]
    fn the_wire_form_carries_the_rule_that_produced_it() {
        let s = score(&[core(2), modality(1)]);
        let w = to_wire(&s);
        assert_eq!(wire_rule_version(&w), RULE_VERSION);
        assert_eq!(w[2], s.brs);
        assert_eq!(w.len(), 8);
    }

    #[test]
    fn the_wire_form_is_deterministic() {
        let s = score(&[core(2), modality(1), neuro()]);
        assert_eq!(to_wire(&s), to_wire(&s));
    }

    #[test]
    fn the_linked_crate_reproduces_its_own_corpus() {
        assert_eq!(
            verify_corpus(),
            None,
            "a pinned vector disagrees at runtime"
        );
    }

    #[test]
    fn a_simulation_reports_the_movement_and_not_just_the_result() {
        let now = [neuro()];
        let proposed = [neuro(), core(1), modality(2)];
        let (after, delta) = simulate(&now, &proposed);
        assert!(after > score(&now).brs);
        assert_eq!(delta, after as i16 - score(&now).brs as i16);
    }

    #[test]
    fn a_simulation_can_report_a_fall() {
        let now = [core(1)];
        let proposed = [core(1), penalty(), penalty()];
        let (_, delta) = simulate(&now, &proposed);
        assert!(delta < 0, "adding penalties must be reportable as a fall");
    }
    #[test]
    fn by_removal_contributions_do_not_sum_to_the_score() {
        // Pinned because it is counter-intuitive and it cost a design. Under a
        // saturating combiner the remaining evidence partly covers for anything
        // removed, so each removal costs more than that evidence's share and
        // the sum overshoots. Anyone rendering by-removal deltas as a column
        // that adds up will be wrong, and this test says so before they try.
        let ev = [core(2), modality(1), neuro()];
        let mut contrib = [Contribution { index: 0, delta: 0 }; 8];
        let n = attribute(&ev, &mut contrib);
        let sum: i16 = contrib[..n].iter().map(|c| c.delta).sum();
        assert_ne!(
            sum,
            score(&ev).brs as i16,
            "if these ever sum, the combiner stopped saturating and this module's \
             two-pass explanation is no longer necessary"
        );
    }

    #[test]
    fn the_waterfall_is_stable_across_runs() {
        let ev = [core(2), modality(2), neuro(), penalty()];
        let run = || {
            let mut s = [Step {
                index: 0,
                kind: Kind::Neuro,
                delta: 0,
                running: 0,
            }; 6];
            let n = explain(&ev, &mut s);
            (n, s)
        };
        assert_eq!(run(), run());
    }
}
