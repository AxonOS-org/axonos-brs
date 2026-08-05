// SPDX-License-Identifier: Apache-2.0 OR MIT
// SPDX-FileCopyrightText: 2026 Denis Yermakou <connect@axonos.org>

//! Conformance corpus — real projects, their evidence, and their pinned scores.
//!
//! Public rather than test-only, and the reason is a defect this file exists to
//! prevent. The corpus lived in `tests/`, so anything outside the test harness
//! that needed it — an example, a cross-target comparison, a third party
//! implementing this combiner — had to copy it. A copy was made, and it was
//! wrong within the hour: one project's evidence was transcribed with three
//! items instead of four, producing a score of 90 where the pinned value is 93.
//!
//! Hand-transcribed vectors diverge. That is not a lapse to be more careful
//! about, it is a property of copying, and the fix is to have one copy. An
//! implementer of this rule in another language can now read the corpus from
//! the crate rather than from a document.

use crate::{Evidence, Kind};

/// One project's evidence and the score this combiner must produce for it.
#[derive(Debug)]
pub struct Vector {
    /// The repository, for a failure message that names what diverged.
    pub name: &'static str,
    /// The evidence exactly as the scanner emitted it.
    pub evidence: &'static [Evidence],
    /// The published score. A change here has to be argued for in a diff.
    pub expect: u8,
}

/// Twenty projects from the live corpus on 2026-07-29, each with the evidence
/// the scanner emitted and the score this combiner must produce.
///
/// Pinning them is what stops the rule drifting silently: a change that moves
/// any of these numbers has to be argued for in a diff rather than discovered
/// later in a chart.
pub const CORPUS: &[Vector] = &[
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
