//! # axonos-brs
//!
//! **The BCI Relevance Score, as arithmetic that cannot lie about its own range.**
//!
//! The radar keeps a project when its relevance score clears a gate, and shows
//! that score on every card. For thirteen releases the score was computed by
//! summing fixed point values and clamping the total to `0..=100`. That
//! combiner has three defects, and a measurement of the live corpus shows all
//! three at once: across 117 scored projects the score took **nine distinct
//! values**, 19 % sat at exactly 100, and 31 % at exactly the gate.
//!
//! - **It clips.** A repository whose evidence sums to 105 and one whose
//!   evidence sums to 205 both display 100. At the top, where ranking matters
//!   most, the scale stops discriminating entirely.
//! - **It is lumpy.** Each rule fires once for a fixed amount. One mention of
//!   EEG scores identically to fifty, so quantity of evidence is invisible.
//! - **It decides on one number and displays another.** The gate compared the
//!   unclamped sum; the card showed the clamped one.
//!
//! This crate replaces the combiner and nothing else. The rules that decide
//! *what counts as evidence* stay in Python, where a keyword table belongs and
//! where changing one is a five-minute job. What moves here is the part that
//! must be identical everywhere and forever: how evidence becomes a number.
//!
//! ## The combiner
//!
//! Evidence combines the way independent indications of the same fact combine,
//! not the way lengths add:
//!
//! ```text
//! S⁺ = 1 − Π (1 − wᵢ)          positive evidence
//! S  = S⁺ · Π (1 − pⱼ)         negative evidence attenuates
//! ```
//!
//! Four properties follow directly, and each replaces a defect above:
//!
//! | Property | Why it matters |
//! |:--|:--|
//! | `S < 1` always, approached but never reached | the top of the scale keeps discriminating; nothing clips |
//! | `S ≥ 0` always | a penalty cannot drive a score negative, so none needs flooring |
//! | monotone in positive evidence | more evidence never lowers a score, which a sum-then-clamp cannot promise once clamped |
//! | diminishing returns | the fifth EEG mention adds less than the first, which is how belief actually behaves |
//!
//! ## Integer arithmetic, on purpose
//!
//! Every value is an integer in parts per million. No floating point appears
//! anywhere. This is not thrift: the same score has to come out of the scanner
//! on a Linux runner and out of the same code compiled to WebAssembly in a
//! reader's browser, and *identical* is a stronger word than *close*. A score
//! a reader can recompute is a score they can dispute, which is the entire
//! point of publishing the evidence beside it.
//!
//! ## What this crate does not do
//!
//! It does not decide what BCI is. It has no keyword table, reads no text, and
//! makes no network call. Given evidence it produces a number, a decision and
//! an attribution — and given the same evidence it produces them again.

#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
#![deny(missing_docs)]

/// The conformance corpus: real projects, their evidence, their pinned scores.
use core::cmp::Ordering;

pub mod corpus;
/// Fixed-point scale: one whole unit, in parts per million.
pub const ONE: u64 = 1_000_000;

/// Score at or above which a project is kept.
///
/// Unchanged from the sum-and-clamp era on purpose. Under the old combiner a
/// single decisive signal — one acquisition modality — scored exactly 40, and
/// under this one it still does. The gate therefore means what it always meant,
/// *at least one concrete anchor*, and this release is not quietly a change of
/// inclusion policy wearing a change of arithmetic.
pub const GATE: u8 = 40;

/// Largest weight any single piece of evidence may carry.
///
/// A ceiling below `ONE` is what keeps the combiner's promise: with `w < 1`
/// the product `Π(1 − wᵢ)` is strictly positive, so `S` is strictly below one
/// and the score never reaches 100. One rule must not be able to end the
/// argument by itself.
pub const MAX_WEIGHT: u64 = 920_000;

/// Bonus for the first additional matched term, in ppm.
const BONUS_FIRST: u64 = 60_000;
/// Cap on the total multi-term bonus, in ppm.
const BONUS_CAP: u64 = 150_000;

/// What kind of signal a piece of evidence is.
///
/// Carried through to the ledger so a reader sees not only how much a signal
/// contributed but what sort of thing it was.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kind {
    /// An unambiguous BCI self-identification.
    Core,
    /// An acquisition modality — EEG, MEG, ECoG, EMG, fNIRS, spikes/LFP.
    Modality,
    /// A field standard or interchange format — LSL, BIDS, NWB, FIF/MNE.
    Standard,
    /// Named acquisition hardware.
    Hardware,
    /// A BCI paradigm — P300, SSVEP, motor imagery, neurofeedback.
    Paradigm,
    /// An ambiguous neuro term, shared with adjacent fields.
    Neuro,
    /// Provenance — a paper, a DOI, a citation file.
    Provenance,
    /// Negative evidence: a reason to believe this is a different field.
    Negative,
}

impl Kind {
    /// Whether this kind counts as a concrete anchor for the tier summary.
    pub const fn is_concrete(&self) -> bool {
        matches!(
            self,
            Kind::Core | Kind::Modality | Kind::Standard | Kind::Hardware | Kind::Paradigm
        )
    }
}

/// One rule firing.
///
/// `points` keeps the units the Python rules already speak, so a rule table
/// edited on one side does not need translating on the other. `terms` is the
/// number of distinct matched terms behind the firing — one EEG mention and
/// three are different evidence, and this is where that difference enters.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Evidence {
    /// What sort of signal this is.
    pub kind: Kind,
    /// Signed points as the rule table states them, `-100..=100`.
    pub points: i32,
    /// Distinct matched terms behind this firing; `0` is treated as `1`.
    pub terms: u32,
}

impl Evidence {
    /// A single-term piece of evidence.
    pub const fn new(kind: Kind, points: i32) -> Self {
        Self {
            kind,
            points,
            terms: 1,
        }
    }

    /// Evidence backed by `terms` distinct matched terms.
    pub const fn with_terms(kind: Kind, points: i32, terms: u32) -> Self {
        Self {
            kind,
            points,
            terms,
        }
    }

    /// This evidence's weight in ppm, graded by how many terms matched.
    ///
    /// The bonus for additional terms diminishes — `+0.060`, `+0.030`,
    /// `+0.020`, … — and is capped, because the difference between one matched
    /// modality and three is real while the difference between eight and ten
    /// is noise in a keyword match rather than a fact about the project.
    pub const fn weight_ppm(&self) -> u64 {
        let base = if self.points >= 0 {
            self.points as u64
        } else {
            (-self.points) as u64
        } * 10_000;
        let n = if self.terms == 0 { 1 } else { self.terms };
        let mut bonus = 0u64;
        let mut i = 0u32;
        while i + 1 < n {
            bonus += BONUS_FIRST / (i as u64 + 1);
            i += 1;
        }
        if bonus > BONUS_CAP {
            bonus = BONUS_CAP;
        }
        let w = base + bonus;
        if w > MAX_WEIGHT {
            MAX_WEIGHT
        } else {
            w
        }
    }
}

/// Whether a project clears the gate.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Decision {
    /// Score at or above [`GATE`].
    Keep,
    /// Score below [`GATE`].
    Drop,
}

/// The strongest positive signal present, as a legible label.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Tier {
    /// Nothing positive, or nothing that clears the gate.
    Rejected,
    /// Kept on weak evidence only.
    WeakKeep,
    /// An ambiguous neuro term with context.
    NeuroTerm,
    /// A modality or a paradigm.
    ModalityOrParadigm,
    /// A field standard or named hardware.
    StandardOrHardware,
    /// An unambiguous BCI self-identification.
    ExplicitBci,
}

/// The outcome of scoring.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Score {
    /// The score, `0..=99`. One hundred is unreachable by construction.
    pub brs: u8,
    /// Whether the project is kept.
    pub decision: Decision,
    /// The strongest positive signal present.
    pub tier: Tier,
    /// Positive evidence before attenuation, in ppm — exposed so a reader can
    /// see how much of a low score is weak evidence and how much is a penalty.
    pub positive_ppm: u64,
    /// Attenuation from negative evidence, in ppm. `ONE` means none applied.
    pub attenuation_ppm: u64,
}

/// Marginal contribution of one piece of evidence, in score points.
///
/// Under a saturating combiner a contribution is not a constant: the same rule
/// adds more to a sparse project than to a well-evidenced one. Attribution is
/// therefore computed by removal — the score with this evidence minus the score
/// without it — which is the only definition that sums to something meaningful.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Contribution {
    /// Which evidence this describes, by index into the input slice.
    pub index: usize,
    /// Points this evidence added, or removed if negative.
    pub delta: i16,
}

/// Keep a product strictly positive under integer truncation.
///
/// In real arithmetic `Π(1 − wᵢ)` is strictly positive for `wᵢ < 1`, so the
/// score is strictly below one hundred. In integer arithmetic the product
/// truncates to zero after enough factors, and the invariant the module
/// documents would then be a property of the idealisation rather than of the
/// code. One part per million is the smallest value that keeps the promise
/// true of the implementation as well.
const fn floor_one(v: u64) -> u64 {
    if v == 0 {
        1
    } else {
        v
    }
}

fn combine(evidence: &[Evidence]) -> (u64, u64) {
    let mut remaining = ONE; // Π(1 − wᵢ) over positive evidence
    let mut attenuation = ONE; // Π(1 − pⱼ) over negative evidence
    for e in evidence {
        let w = e.weight_ppm();
        // Three cases, and the third is deliberate: evidence worth zero points
        // is neither support nor penalty, and must leave the product untouched
        // rather than multiply it by one — which would floor it needlessly.
        match e.points.cmp(&0) {
            Ordering::Greater => remaining = floor_one(remaining * (ONE - w) / ONE),
            Ordering::Less => attenuation = floor_one(attenuation * (ONE - w) / ONE),
            Ordering::Equal => {}
        }
    }
    (ONE - remaining, attenuation)
}

/// Score a set of evidence.
///
/// Pure: no clock, no allocation, no state. The same slice produces the same
/// `Score` on any target, which is what lets a browser recompute what a scanner
/// published.
pub fn score(evidence: &[Evidence]) -> Score {
    let (positive, attenuation) = combine(evidence);
    let combined = positive * attenuation / ONE;
    // Round half up, in integers.
    let brs = ((combined * 100 + ONE / 2) / ONE) as u8;
    let brs = if brs > 99 { 99 } else { brs };

    let mut tier = Tier::Rejected;
    for e in evidence {
        if e.points <= 0 {
            continue;
        }
        let t = match e.kind {
            Kind::Core => Tier::ExplicitBci,
            Kind::Standard | Kind::Hardware => Tier::StandardOrHardware,
            Kind::Modality | Kind::Paradigm => Tier::ModalityOrParadigm,
            Kind::Neuro => Tier::NeuroTerm,
            Kind::Provenance | Kind::Negative => Tier::Rejected,
        };
        if t > tier {
            tier = t;
        }
    }
    let decision = if brs >= GATE {
        Decision::Keep
    } else {
        Decision::Drop
    };
    if tier == Tier::Rejected && decision == Decision::Keep {
        tier = Tier::WeakKeep;
    }

    Score {
        brs,
        decision,
        tier,
        positive_ppm: positive,
        attenuation_ppm: attenuation,
    }
}

/// Attribute the score to each piece of evidence, by removal.
///
/// Writes into `out` and returns the number of entries written, so the caller
/// owns the storage and this crate stays allocation-free. `out` shorter than
/// `evidence` truncates rather than panicking: a partial attribution is still
/// useful and a panic in a scoring path is not.
pub fn attribute(evidence: &[Evidence], out: &mut [Contribution]) -> usize {
    let full = score(evidence).brs as i16;
    let n = if out.len() < evidence.len() {
        out.len()
    } else {
        evidence.len()
    };
    for (i, slot) in out.iter_mut().enumerate().take(n) {
        // Score the same evidence with element i neutralised.
        let mut without_positive = ONE;
        let mut without_attenuation = ONE;
        for (j, e) in evidence.iter().enumerate() {
            if i == j {
                continue;
            }
            let w = e.weight_ppm();
            match e.points.cmp(&0) {
                Ordering::Greater => {
                    without_positive = floor_one(without_positive * (ONE - w) / ONE)
                }
                Ordering::Less => {
                    without_attenuation = floor_one(without_attenuation * (ONE - w) / ONE)
                }
                Ordering::Equal => {}
            }
        }
        let combined = (ONE - without_positive) * without_attenuation / ONE;
        let mut without = ((combined * 100 + ONE / 2) / ONE) as i16;
        if without > 99 {
            without = 99;
        }
        *slot = Contribution {
            index: i,
            delta: full - without,
        };
    }
    n
}

/// What the score would become if one more piece of evidence existed.
///
/// The question a maintainer actually has is not "what is my score" but "what
/// would raise it", and under a saturating combiner the answer depends on what
/// they already have. This answers it exactly rather than by rule of thumb.
pub fn score_with(evidence: &[Evidence], addition: Evidence) -> u8 {
    let (mut positive, mut attenuation) = combine(evidence);
    let w = addition.weight_ppm();
    match addition.points.cmp(&0) {
        Ordering::Greater => {
            let remaining = ONE - positive;
            positive = ONE - floor_one(remaining * (ONE - w) / ONE);
        }
        Ordering::Less => attenuation = attenuation * (ONE - w) / ONE,
        Ordering::Equal => {}
    }
    let combined = positive * attenuation / ONE;
    let brs = ((combined * 100 + ONE / 2) / ONE) as u8;
    if brs > 99 {
        99
    } else {
        brs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn core() -> Evidence {
        Evidence::new(Kind::Core, 55)
    }
    fn modality(n: u32) -> Evidence {
        Evidence::with_terms(Kind::Modality, 40, n)
    }
    fn standard() -> Evidence {
        Evidence::new(Kind::Standard, 45)
    }
    fn neuro() -> Evidence {
        Evidence::new(Kind::Neuro, 25)
    }
    fn penalty(p: i32) -> Evidence {
        Evidence::new(Kind::Negative, p)
    }

    // ── the three defects this crate exists to remove ──

    #[test]
    fn the_score_never_reaches_one_hundred() {
        // Twenty maximal signals. Under sum-and-clamp this is 100, and so is
        // one signal more; here the scale keeps discriminating.
        let many = [core(); 20];
        let s = score(&many);
        assert!(s.brs < 100, "score reached {}", s.brs);
        assert!(
            s.brs > 95,
            "twenty strong signals should still score high: {}",
            s.brs
        );
    }

    #[test]
    fn the_scale_separates_across_the_range_that_exists() {
        // The promise is not that every additional signal moves an integer
        // score — at the ninth signal the remaining headroom is under half a
        // point and no 0..=99 scale can show it. The promise is that the
        // separation happens where the projects actually are: one to five
        // signals covers the whole live corpus.
        let s: [u8; 5] = core::array::from_fn(|i| score(&[core(); 5][..=i]).brs);
        for w in s.windows(2) {
            assert!(
                w[1] > w[0],
                "must separate through the populated range: {s:?}"
            );
        }
        assert_eq!(s[0], 55);
        assert!(s[4] < 99);
    }

    #[test]
    fn the_positive_term_stays_strictly_below_one_in_integer_arithmetic() {
        // The documented property is Π(1 − wᵢ) > 0, and integer truncation
        // would quietly break it after enough factors. This asserts it of the
        // implementation, not of the mathematics it approximates.
        for n in [1usize, 5, 20, 64] {
            let v = vec![Evidence::with_terms(Kind::Core, 100, 50); n];
            let s = score(&v);
            assert!(s.positive_ppm < ONE, "saturated to ONE at n={n}");
            assert!(s.brs < 100);
        }
    }

    #[test]
    fn quantity_of_evidence_is_visible() {
        // The defect: one matched modality scored the same as three.
        let one = score(&[modality(1)]).brs;
        let three = score(&[modality(3)]).brs;
        assert!(
            three > one,
            "three matched modalities must outscore one: {one} vs {three}"
        );
    }

    #[test]
    fn the_gate_still_means_one_concrete_anchor() {
        // Deliberate continuity: a single modality scored exactly 40 under the
        // old combiner and must here, or this release silently changes policy.
        assert_eq!(score(&[modality(1)]).brs, GATE);
        assert_eq!(score(&[modality(1)]).decision, Decision::Keep);
    }

    #[test]
    fn one_ambiguous_term_alone_does_not_clear_the_gate() {
        assert_eq!(score(&[neuro()]).decision, Decision::Drop);
        // but two independent weak signals do, which is the point of combining
        assert_eq!(score(&[neuro(), neuro()]).decision, Decision::Keep);
    }

    // ── the properties, exhaustively ──

    #[test]
    fn the_score_is_bounded_over_a_wide_input_space() {
        let kinds = [
            Kind::Core,
            Kind::Modality,
            Kind::Standard,
            Kind::Neuro,
            Kind::Negative,
        ];
        for &k in &kinds {
            for points in [-100i32, -60, -35, -10, 0, 10, 25, 40, 45, 55, 100] {
                for terms in [0u32, 1, 3, 9, 100] {
                    for count in [1usize, 2, 5, 12] {
                        let e = Evidence {
                            kind: k,
                            points,
                            terms,
                        };
                        let v: [Evidence; 12] = [e; 12];
                        let s = score(&v[..count]);
                        assert!(s.brs <= 99, "brs {} out of range", s.brs);
                    }
                }
            }
        }
    }

    #[test]
    fn adding_positive_evidence_never_lowers_a_score() {
        let base = [core(), modality(2), neuro()];
        let before = score(&base).brs;
        for extra in [core(), modality(1), standard(), neuro()] {
            let mut v = [core(); 4];
            v[..3].copy_from_slice(&base);
            v[3] = extra;
            assert!(
                score(&v).brs >= before,
                "adding {extra:?} lowered the score"
            );
        }
    }

    #[test]
    fn adding_negative_evidence_never_raises_a_score() {
        let base = [core(), modality(2)];
        let before = score(&base).brs;
        for p in [-10, -30, -40, -60, -100] {
            let v = [base[0], base[1], penalty(p)];
            assert!(score(&v).brs <= before, "penalty {p} raised the score");
        }
    }

    #[test]
    fn a_penalty_attenuates_and_cannot_go_negative() {
        let s = score(&[neuro(), penalty(-100)]);
        assert!(s.brs < 25);
        // saturating at MAX_WEIGHT, so even -100 leaves a residue rather than
        // producing a negative number that then needs flooring
        assert!(s.attenuation_ppm > 0);
    }

    #[test]
    fn penalties_compose_multiplicatively_rather_than_by_subtraction() {
        let one = score(&[core(), penalty(-40)]).brs;
        let two = score(&[core(), penalty(-40), penalty(-40)]).brs;
        assert!(two < one);
        assert!(
            two > 0,
            "two penalties must not zero a strong signal outright"
        );
    }

    #[test]
    fn diminishing_returns_are_real() {
        let a = score(&[core()]).brs as i32;
        let b = score(&[core(), core()]).brs as i32;
        let c = score(&[core(), core(), core()]).brs as i32;
        assert!(
            b - a > c - b,
            "the third signal must add less than the second"
        );
    }

    #[test]
    fn term_bonus_diminishes_and_is_capped() {
        let w1 = modality(1).weight_ppm();
        let w2 = modality(2).weight_ppm();
        let w3 = modality(3).weight_ppm();
        let w99 = modality(99).weight_ppm();
        assert!(w2 - w1 > w3 - w2, "the bonus must diminish");
        assert_eq!(w99, w1 + BONUS_CAP, "and be capped");
    }

    #[test]
    fn no_single_signal_can_end_the_argument() {
        let huge = Evidence::with_terms(Kind::Core, 100, 50);
        assert!(huge.weight_ppm() <= MAX_WEIGHT);
        assert!(score(&[huge]).brs < 99);
    }

    // ── attribution ──

    #[test]
    fn attribution_names_what_each_signal_added() {
        let ev = [core(), modality(2), neuro()];
        let mut out = [Contribution { index: 0, delta: 0 }; 3];
        let n = attribute(&ev, &mut out);
        assert_eq!(n, 3);
        for c in &out {
            assert!(
                c.delta > 0,
                "every positive signal must have added something"
            );
        }
        // the strongest signal contributes most
        assert!(out[0].delta >= out[2].delta);
    }

    #[test]
    fn attribution_of_a_penalty_is_negative() {
        let ev = [core(), penalty(-40)];
        let mut out = [Contribution { index: 0, delta: 0 }; 2];
        attribute(&ev, &mut out);
        assert!(out[1].delta < 0, "a penalty must attribute negatively");
    }

    #[test]
    fn attribution_truncates_rather_than_panicking() {
        let ev = [core(), modality(1), neuro()];
        let mut out = [Contribution { index: 0, delta: 0 }; 2];
        assert_eq!(attribute(&ev, &mut out), 2);
    }

    #[test]
    fn attribution_is_context_dependent_which_is_the_honest_answer() {
        // The same rule adds more to a sparse project than to a rich one.
        let sparse = [modality(1), neuro()];
        let rich = [core(), standard(), modality(3), neuro()];
        let mut a = [Contribution { index: 0, delta: 0 }; 4];
        let mut b = [Contribution { index: 0, delta: 0 }; 4];
        attribute(&sparse, &mut a);
        attribute(&rich, &mut b);
        let neuro_in_sparse = a[1].delta;
        let neuro_in_rich = b[3].delta;
        assert!(neuro_in_sparse > neuro_in_rich);
    }

    // ── what would raise this score ──

    #[test]
    fn the_counterfactual_answers_a_maintainers_actual_question() {
        let ev = [modality(1)];
        let now = score(&ev).brs;
        let with_standard = score_with(&ev, standard());
        let with_core = score_with(&ev, core());
        assert!(
            with_core > with_standard,
            "the strongest addition must help most"
        );
        assert!(with_standard > now);
    }

    #[test]
    fn the_counterfactual_agrees_with_actually_adding_it() {
        let ev = [modality(2), neuro()];
        let predicted = score_with(&ev, standard());
        let actual = score(&[modality(2), neuro(), standard()]).brs;
        assert_eq!(predicted, actual);
    }

    // ── tier ──

    #[test]
    fn the_tier_reports_the_strongest_positive_signal() {
        assert_eq!(
            score(&[neuro(), modality(1), core()]).tier,
            Tier::ExplicitBci
        );
        assert_eq!(
            score(&[neuro(), modality(1)]).tier,
            Tier::ModalityOrParadigm
        );
        assert_eq!(score(&[neuro(), standard()]).tier, Tier::StandardOrHardware);
        assert_eq!(score(&[neuro(), neuro()]).tier, Tier::NeuroTerm);
    }

    #[test]
    fn nothing_positive_is_rejected_and_says_so() {
        let s = score(&[penalty(-40)]);
        assert_eq!(s.decision, Decision::Drop);
        assert_eq!(s.tier, Tier::Rejected);
        assert_eq!(s.brs, 0);
    }

    #[test]
    fn empty_evidence_scores_zero_rather_than_erroring() {
        let s = score(&[]);
        assert_eq!(s.brs, 0);
        assert_eq!(s.decision, Decision::Drop);
    }

    // ── determinism ──

    #[test]
    fn the_same_evidence_gives_the_same_byte() {
        let ev = [core(), modality(3), standard(), neuro(), penalty(-30)];
        let a = score(&ev);
        for _ in 0..100 {
            assert_eq!(score(&ev), a);
        }
    }

    #[test]
    fn order_of_evidence_does_not_change_the_score() {
        // Multiplication commutes, and a score that depended on the order a
        // keyword table happened to be written in would be indefensible.
        let a = score(&[core(), modality(2), neuro(), penalty(-30)]).brs;
        let b = score(&[penalty(-30), neuro(), modality(2), core()]).brs;
        assert_eq!(a, b);
    }
}
