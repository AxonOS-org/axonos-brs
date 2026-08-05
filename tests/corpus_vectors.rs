//! The conformance corpus, exercised against the combiner.
//!
//! The vectors themselves live in `src/corpus.rs` and are public: anything
//! that needs them — this suite, the cross-target comparison, a third party
//! implementing the rule elsewhere — reads the same list. A copy was made once
//! and was wrong within the hour.
//!
//! Each is a real project's evidence as the scanner emitted it. Pinning
//! them is what stops the scoring rule from drifting silently: a change
//! that moves any of these numbers has to be argued for in a diff, not
//! discovered later in a chart.

use axonos_brs::*;

use axonos_brs::corpus::CORPUS as VECTORS;

/// Every vector reproduces its pinned score.
///
/// Run with `expect: 0` on a fresh vector set and this test prints the values
/// to paste in; from then on it is a lock.
#[test]
fn the_corpus_vectors_reproduce_their_pinned_scores() {
    let mut drift = Vec::new();
    for v in VECTORS {
        let got = score(v.evidence).brs;
        if got != v.expect {
            drift.push(format!("  {} → {} (pinned {})", v.name, got, v.expect));
        }
    }
    assert!(
        drift.is_empty(),
        "the scoring rule moved on {} of {} pinned vectors:\n{}",
        drift.len(),
        VECTORS.len(),
        drift.join("\n")
    );
}

/// The defects this crate was built to remove, measured on the real corpus
/// rather than on invented inputs.
#[test]
fn the_corpus_is_no_longer_clumped() {
    let scores: Vec<u8> = VECTORS.iter().map(|v| score(v.evidence).brs).collect();
    let mut distinct: Vec<u8> = scores.clone();
    distinct.sort_unstable();
    distinct.dedup();

    assert!(
        !scores.iter().any(|&s| s >= 100),
        "nothing may sit at the ceiling"
    );
    // The old combiner produced nine distinct values across the whole corpus.
    assert!(
        distinct.len() * 2 >= scores.len(),
        "resolution too low: {} distinct across {} projects",
        distinct.len(),
        scores.len()
    );
}

/// Every kept project can be told why, and every rejected one too.
#[test]
fn every_vector_attributes_completely() {
    for v in VECTORS {
        let mut out = vec![Contribution { index: 0, delta: 0 }; v.evidence.len()];
        let n = attribute(v.evidence, &mut out);
        assert_eq!(n, v.evidence.len(), "{}", v.name);
        let positives = v.evidence.iter().filter(|e| e.points > 0).count();
        let attributed = out.iter().filter(|c| c.delta > 0).count();
        assert!(
            attributed <= positives,
            "{}: attributed more positives than exist",
            v.name
        );
    }
}
