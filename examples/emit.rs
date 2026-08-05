// SPDX-License-Identifier: Apache-2.0 OR MIT
// SPDX-FileCopyrightText: 2026 Denis Yermakou <connect@axonos.org>
//
// Part of axonos-brs. Dual-licensed Apache-2.0 OR MIT at your option.
// Authored by Denis Yermakou for The AxonOS Project — https://axonos.org

//! Emit every corpus score in a fixed, greppable form.
//!
//! The README says the same score must come out of the scanner on a Linux
//! runner and out of the same code compiled to WebAssembly in a reader's
//! browser, and that *identical* is a stronger word than *close*. That was a
//! statement of intent with nothing behind it: nothing in this repository
//! built the WebAssembly target, let alone compared the two.
//!
//! This program is the comparison. CI builds it twice — once for the host and
//! once for `wasm32-wasip1` — runs both, and diffs the output. A single
//! differing byte fails the build, which is what the claim is worth if it is
//! worth anything.
//!
//! Why `wasm32-wasip1` rather than the browser's `wasm32-unknown-unknown`: the
//! browser target has no way to print, so it cannot be diffed standalone. The
//! two targets share the same code generation for the arithmetic that matters
//! here — this proves the integers agree across a target boundary, and CI
//! separately builds the browser `cdylib` to prove that target compiles at all.
//! Neither alone would be enough; together they are what the README claims.

use axonos_brs::{corpus::CORPUS, score};

fn main() {
    // One line per project, every field that the arithmetic produces. Fields
    // beyond `brs` are included on purpose: a divergence in the intermediate
    // ppm figures that happened to round to the same published score would be
    // a real difference between the two targets, and printing only the score
    // would hide exactly the case worth catching.
    for v in CORPUS {
        let (name, s) = (v.name, score(v.evidence));
        println!(
            "{name}\tbrs={}\tdecision={:?}\ttier={:?}\tpositive_ppm={}",
            s.brs, s.decision, s.tier, s.positive_ppm
        );
    }
}
