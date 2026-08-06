// SPDX-License-Identifier: Apache-2.0 OR MIT
// SPDX-FileCopyrightText: 2026 Denis Yermakou <connect@axonos.org>
//
// Part of axonos-brs. Dual-licensed Apache-2.0 OR MIT at your option.
// Authored by Denis Yermakou for The AxonOS Project — https://axonos.org

//! The browser ABI for [`axonos_brs`].
//!
//! The README has claimed since the first release that a reader's browser can
//! recompute a published score from the same code the scanner uses. Until this
//! module existed, that was true of the *arithmetic* and not of the artifact:
//! `--crate-type cdylib` for `wasm32-unknown-unknown` refused to link, because a
//! `no_std` dynamic library has to supply its own panic handler and this one
//! supplied none. The claim was one linker error away from being false.
//!
//! ## Why this is a separate crate
//!
//! `axonos-brs` carries `#![forbid(unsafe_code)]`, and that is not decoration:
//! it is most of the reason the arithmetic is worth trusting. Exporting a symbol
//! under a name you chose is an unsafe operation — the linker's behaviour with
//! duplicate names is undefined — so `#[no_mangle]` is refused there, correctly.
//!
//! Weakening the forbiddance to make a binding fit would have traded a headline
//! property for a convenience. The binding lives here instead, where FFI is the
//! declared business of the crate and the scoring code is a dependency it cannot
//! reach into.
//!
//! ## Why the ABI takes arguments instead of a pointer
//!
//! The obvious shape is a pointer and a length: JavaScript writes evidence into
//! the module's linear memory and Rust reads it back. That needs raw pointer
//! dereferences, and this crate is `#![forbid(unsafe_code)]`. The forbiddance is
//! not decoration — it is most of why the arithmetic is worth trusting — so the
//! ABI bends around it rather than the other way.
//!
//! It bends cleanly because of a fact about the rules: **each rule fires at most
//! once**, and there are eight kinds. A ledger therefore holds at most eight
//! entries, so eight arguments is not a limit imposed for convenience, it is the
//! actual maximum. A ninth would mean a rule fired twice, which the scanner
//! cannot do.
//!
//! ## The packing
//!
//! Each argument is one evidence item in a `u32`, or zero for an empty slot:
//!
//! ```text
//! bits  0..=3    kind      0=Core 1=Modality 2=Standard 3=Hardware
//!                          4=Paradigm 5=Neuro 6=Provenance 7=Negative
//! bits  4..=11   points    signed, offset by 128 (so 0 points encodes as 128)
//! bits 12..=19   terms     1..=255, clamped
//! bit  20        present   must be 1; a zero argument is an empty slot
//! ```
//!
//! An item with `present` clear is skipped, so a caller with three signals
//! passes three packed values and five zeros.
//!
//! ## Calling it
//!
//! ```js
//! const { instance } = await WebAssembly.instantiateStreaming(fetch("axonos_brs.wasm"));
//! const pack = (kind, points, terms) =>
//!   (1 << 20) | ((terms & 0xff) << 12) | (((points + 128) & 0xff) << 4) | (kind & 0xf);
//! // Core, 55 points, 2 matched terms
//! instance.exports.brs_score(pack(0, 55, 2), 0, 0, 0, 0, 0, 0, 0);  // → 61
//! ```
//!
//! Check `brs_rule_version()` against the `rule` field of a published score
//! before comparing them. Two numbers produced by different rules are not
//! comparable, and the cheapest place to notice that is before the comparison.

#![no_std]

use axonos_brs::{score, Evidence, Kind};

/// Bit marking a slot as carrying evidence.
const PRESENT: u32 = 1 << 20;

/// Offset applied to the points field so a negative value packs into `u8`.
const POINTS_BIAS: i32 = 128;

fn unpack(v: u32) -> Option<Evidence> {
    if v & PRESENT == 0 {
        return None;
    }
    let kind = match v & 0xF {
        0 => Kind::Core,
        1 => Kind::Modality,
        2 => Kind::Standard,
        3 => Kind::Hardware,
        4 => Kind::Paradigm,
        5 => Kind::Neuro,
        6 => Kind::Provenance,
        7 => Kind::Negative,
        // A kind this crate does not define is refused rather than mapped to a
        // neighbour. A caller sending 9 has a bug, and scoring it as Negative
        // would hide the bug behind a plausible number.
        _ => return None,
    };
    let points = ((v >> 4) & 0xFF) as i32 - POINTS_BIAS;
    let terms = ((v >> 12) & 0xFF).max(1);
    Some(Evidence::with_terms(kind, points, terms))
}

/// Score up to eight packed evidence items. Returns `0..=99`.
///
/// Eight is the whole ledger, not a truncation: one entry per rule, eight rules.
#[no_mangle]
pub extern "C" fn brs_score(
    e0: u32,
    e1: u32,
    e2: u32,
    e3: u32,
    e4: u32,
    e5: u32,
    e6: u32,
    e7: u32,
) -> u32 {
    let packed = [e0, e1, e2, e3, e4, e5, e6, e7];
    let mut items = [Evidence::new(Kind::Neuro, 0); 8];
    let mut n = 0;
    for p in packed.iter() {
        if let Some(e) = unpack(*p) {
            items[n] = e;
            n += 1;
        }
    }
    score(&items[..n]).brs as u32
}

/// The rule that `brs_score` implements.
///
/// A published score outlives the release that computed it. Comparing a number
/// from the map against one from this module without checking that both came
/// from the same rule compares two different things and reports a difference in
/// the project.
#[no_mangle]
pub extern "C" fn brs_rule_version() -> u32 {
    axonos_brs::analysis::RULE_VERSION as u32
}

/// The inclusion threshold, so a caller need not hardcode it.
#[no_mangle]
pub extern "C" fn brs_gate() -> u32 {
    axonos_brs::GATE as u32
}

/// Panic handler for the standalone browser artifact.
///
/// Required by the target: a `no_std` dynamic library has no host runtime to
/// fall back on, and without this the link fails with `#[panic_handler] function
/// required, but not found` — which is exactly how CI found out that the
/// browser claim had never been built.
///
/// Not gated on the target. This crate builds only as a `cdylib`, so the
/// handler is required on every target it can be built for, and a `cdylib` is
/// never linked into someone else's program the way an `rlib` is — there is
/// nothing to collide with. Ungating it also means the crate compiles on a
/// development machine, which is how this file got checked before it was
/// pushed rather than after.
///
/// The body is a loop rather than an abort, because aborting needs `unsafe` and
/// this crate forbids it. Reaching here would mean the arithmetic panicked,
/// which it is written not to do: every operation saturates or is checked, and
/// the conformance corpus exercises the boundaries. A hang is a poor outcome and
/// a wrong answer is a worse one.
#[cfg(not(test))]
#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use axonos_brs::GATE;

    fn pack(kind: u32, points: i32, terms: u32) -> u32 {
        PRESENT | ((terms & 0xFF) << 12) | ((((points + POINTS_BIAS) as u32) & 0xFF) << 4) | kind
    }

    #[test]
    fn a_packed_item_round_trips() {
        let e = unpack(pack(0, 55, 2)).expect("present");
        assert_eq!(e.kind, Kind::Core);
        assert_eq!(e.points, 55);
        assert_eq!(e.terms, 2);
    }

    #[test]
    fn negative_points_survive_the_bias() {
        let e = unpack(pack(7, -40, 1)).expect("present");
        assert_eq!(e.kind, Kind::Negative);
        assert_eq!(e.points, -40);
    }

    #[test]
    fn an_empty_slot_is_skipped_not_scored() {
        assert!(unpack(0).is_none());
        // Eight empty slots is an empty ledger, and an empty ledger scores zero
        // rather than erroring: a project with no evidence is a real case.
        assert_eq!(brs_score(0, 0, 0, 0, 0, 0, 0, 0), 0);
    }

    #[test]
    fn an_unknown_kind_is_refused_rather_than_mapped_to_a_neighbour() {
        // Kind 9 does not exist. Scoring it as something else would hide a
        // caller's bug behind a plausible number.
        assert!(unpack(PRESENT | 9).is_none());
    }

    #[test]
    fn terms_of_zero_are_lifted_to_one() {
        // A rule that matched must have matched at least one term; zero is a
        // caller error and one is the only sane reading of it.
        assert_eq!(unpack(pack(0, 55, 0)).expect("present").terms, 1);
    }

    #[test]
    fn the_wasm_entry_agrees_with_the_rust_entry() {
        // The whole point: the browser and the scanner must not disagree.
        for v in axonos_brs::corpus::CORPUS.iter().take(8) {
            let mut packed = [0u32; 8];
            for (slot, e) in packed.iter_mut().zip(v.evidence.iter()) {
                let k = match e.kind {
                    Kind::Core => 0,
                    Kind::Modality => 1,
                    Kind::Standard => 2,
                    Kind::Hardware => 3,
                    Kind::Paradigm => 4,
                    Kind::Neuro => 5,
                    Kind::Provenance => 6,
                    Kind::Negative => 7,
                };
                *slot = pack(k, e.points, e.terms);
            }
            let got = brs_score(
                packed[0], packed[1], packed[2], packed[3], packed[4], packed[5], packed[6],
                packed[7],
            );
            assert_eq!(got as u8, v.expect, "{} disagrees across the ABI", v.name);
        }
    }

    #[test]
    fn eight_slots_is_the_whole_ledger_not_a_truncation() {
        // One entry per rule, eight rules. A ninth would mean a rule fired
        // twice, which the scanner cannot do.
        assert_eq!(
            [
                Kind::Core,
                Kind::Modality,
                Kind::Standard,
                Kind::Hardware,
                Kind::Paradigm,
                Kind::Neuro,
                Kind::Provenance,
                Kind::Negative
            ]
            .len(),
            8
        );
    }

    #[test]
    fn the_exported_constants_match_the_crate() {
        assert_eq!(brs_gate(), GATE as u32);
        assert_eq!(
            brs_rule_version(),
            axonos_brs::analysis::RULE_VERSION as u32
        );
    }
}
