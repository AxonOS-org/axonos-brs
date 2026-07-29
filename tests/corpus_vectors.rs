//! Conformance vectors drawn from the live corpus on 2026-07-29.
//!
//! Each is a real project's evidence as the scanner emitted it. Pinning
//! them is what stops the scoring rule from drifting silently: a change
//! that moves any of these numbers has to be argued for in a diff, not
//! discovered later in a chart.

use axonos_brs::*;

struct Vector {
    name: &'static str,
    evidence: &'static [Evidence],
    expect: u8,
}

const VECTORS: &[Vector] = &[
    Vector {
        name: "AxonOS-org/axonos-protocol",
        evidence: &[Evidence::with_terms(Kind::Core, 55, 1)],
        expect: 55,
    },
    Vector {
        name: "AxonOS-org/axonos-signal-pipeline",
        evidence: &[Evidence::with_terms(Kind::Core, 55, 2)],
        expect: 61,
    },
    Vector {
        name: "BasedHardware/omi",
        evidence: &[Evidence::with_terms(Kind::Core, 55, 1)],
        expect: 55,
    },
    Vector {
        name: "Jalte-Diye-Foundation/NuroLab",
        evidence: &[Evidence::with_terms(Kind::Core, 55, 2)],
        expect: 61,
    },
    Vector {
        name: "JuliaHealth/NeuroAnalyzer.jl",
        evidence: &[Evidence::with_terms(Kind::Modality, 40, 3)],
        expect: 49,
    },
    Vector {
        name: "NsquaredLab/MyoGestic",
        evidence: &[Evidence::with_terms(Kind::Modality, 40, 1)],
        expect: 40,
    },
    Vector {
        name: "NeuroSkill-com/skill",
        evidence: &[
            Evidence::with_terms(Kind::Core, 55, 1),
            Evidence::with_terms(Kind::Modality, 40, 1),
        ],
        expect: 73,
    },
    Vector {
        name: "NeuroSkill-com/vscode-neuroskill",
        evidence: &[
            Evidence::with_terms(Kind::Core, 55, 1),
            Evidence::with_terms(Kind::Modality, 40, 1),
        ],
        expect: 73,
    },
    Vector {
        name: "NeuroTechX/moabb",
        evidence: &[
            Evidence::with_terms(Kind::Core, 55, 2),
            Evidence::with_terms(Kind::Modality, 40, 1),
        ],
        expect: 77,
    },
    Vector {
        name: "Prince445-hub/mlx-drifting-model",
        evidence: &[
            Evidence::with_terms(Kind::Neuro, 25, 1),
            Evidence::with_terms(Kind::Neuro, 15, 1),
        ],
        expect: 36,
    },
    Vector {
        name: "SilverLabUCL/pyNeuroMatic",
        evidence: &[
            Evidence::with_terms(Kind::Neuro, 25, 2),
            Evidence::with_terms(Kind::Neuro, 15, 1),
        ],
        expect: 41,
    },
    Vector {
        name: "SpikeInterface/spikeinterface",
        evidence: &[
            Evidence::with_terms(Kind::Modality, 40, 1),
            Evidence::with_terms(Kind::Neuro, 25, 1),
        ],
        expect: 55,
    },
    Vector {
        name: "Y-Research-SBU/NeuroSonic",
        evidence: &[
            Evidence::with_terms(Kind::Core, 55, 2),
            Evidence::with_terms(Kind::Modality, 40, 1),
        ],
        expect: 77,
    },
    Vector {
        name: "bids-standard/pybv",
        evidence: &[
            Evidence::with_terms(Kind::Modality, 40, 1),
            Evidence::with_terms(Kind::Hardware, 45, 1),
        ],
        expect: 67,
    },
    Vector {
        name: "mne-rt-org/antares",
        evidence: &[
            Evidence::with_terms(Kind::Core, 55, 1),
            Evidence::with_terms(Kind::Modality, 40, 2),
            Evidence::with_terms(Kind::Standard, 45, 1),
            Evidence::with_terms(Kind::Paradigm, 40, 1),
        ],
        expect: 92,
    },
    Vector {
        name: "mne-rt-org/mne-rt",
        evidence: &[
            Evidence::with_terms(Kind::Core, 55, 2),
            Evidence::with_terms(Kind::Modality, 40, 2),
            Evidence::with_terms(Kind::Standard, 45, 1),
            Evidence::with_terms(Kind::Paradigm, 40, 1),
        ],
        expect: 93,
    },
    Vector {
        name: "toniIepure25/Imagina",
        evidence: &[
            Evidence::with_terms(Kind::Core, 55, 2),
            Evidence::with_terms(Kind::Modality, 40, 1),
            Evidence::with_terms(Kind::Standard, 45, 1),
            Evidence::with_terms(Kind::Paradigm, 40, 1),
        ],
        expect: 92,
    },
    Vector {
        name: "enkhbold470/bci-mcp",
        evidence: &[
            Evidence::with_terms(Kind::Core, 55, 2),
            Evidence::with_terms(Kind::Modality, 40, 1),
            Evidence::with_terms(Kind::Standard, 45, 2),
            Evidence::with_terms(Kind::Paradigm, 40, 1),
            Evidence::with_terms(Kind::Neuro, 25, 1),
        ],
        expect: 95,
    },
    Vector {
        name: "josephreggy23-coder/IBL-Brain-Wide-Map",
        evidence: &[
            Evidence::with_terms(Kind::Modality, 40, 1),
            Evidence::with_terms(Kind::Standard, 45, 1),
            Evidence::with_terms(Kind::Hardware, 45, 1),
            Evidence::with_terms(Kind::Neuro, 25, 1),
            Evidence::with_terms(Kind::Neuro, 25, 1),
        ],
        expect: 90,
    },
    Vector {
        name: "ruvnet/ruv-neural",
        evidence: &[
            Evidence::with_terms(Kind::Core, 55, 2),
            Evidence::with_terms(Kind::Modality, 40, 1),
            Evidence::with_terms(Kind::Paradigm, 40, 1),
            Evidence::with_terms(Kind::Neuro, 25, 1),
            Evidence::with_terms(Kind::Neuro, 25, 1),
        ],
        expect: 92,
    },
];

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
