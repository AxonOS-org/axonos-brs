<div align="center">

# axonos-brs

### The relevance score, as arithmetic that cannot lie about its own range.

[![Tests](https://img.shields.io/badge/tests-28%20passing-0d7a5f?style=flat-square)](tests/)
[![no_std](https://img.shields.io/badge/no__std-yes-0a4a8f?style=flat-square)](#constraints)
[![Integer only](https://img.shields.io/badge/floating%20point-none-0a4a8f?style=flat-square)](#integer-arithmetic-on-purpose)
[![Dependencies](https://img.shields.io/badge/dependencies-0-0a4a8f?style=flat-square)](Cargo.toml)
[![License](https://img.shields.io/badge/License-Apache--2.0%20OR%20MIT-475569?style=flat-square)](#licensing)

</div>

---

## The measurement that started this

For thirteen releases the radar computed relevance by summing fixed point
values and clamping the total to `0..=100`. Measured on the live corpus of 117
scored projects, that combiner produced:

| | Before |
|:--|--:|
| distinct score values | **9** |
| projects at exactly 100 | **23** (19 %) |
| projects at exactly the gate | **37** (31 %) |

Nine values is not a scale. It is a category label wearing a number, and it
cannot rank, cannot sort, and cannot answer "is this one better than that one".

## Three defects, one cause

- **It clips.** Evidence summing to 105 and evidence summing to 205 both
  display 100. At the top — where ranking matters most — the scale stops
  discriminating entirely.
- **It is lumpy.** Each rule fires once for a fixed amount, so one mention of
  EEG scores identically to fifty and quantity of evidence is invisible.
- **It decided on one number and displayed another.** The gate compared the
  unclamped sum; the card showed the clamped one.

The cause is the combiner, not the rules. The rules — which disambiguate
neuromorphic ML, cardiac electrophysiology, neuroimaging and generic deep
learning from actual BCI work — are careful and stay exactly where they are.

## What replaces it

Evidence combines the way independent indications of one fact combine, not the
way lengths add:

```text
S⁺ = 1 − Π (1 − wᵢ)      positive evidence
S  = S⁺ · Π (1 − pⱼ)     negative evidence attenuates rather than subtracts
```

| Property | What it fixes |
|:--|:--|
| `S < 1` always | nothing clips; the top of the scale keeps ranking |
| `S ≥ 0` always | a penalty cannot drive a score negative, so none needs flooring |
| monotone in positive evidence | more evidence never lowers a score |
| diminishing returns | the fifth EEG mention adds less than the first |
| commutative | the score cannot depend on the order a keyword table was written in |

Weights are graded by how many distinct terms matched, with a diminishing and
capped bonus — because one matched modality and three are different evidence,
while eight and ten is noise in a keyword match rather than a fact.

### Measured on the same corpus

| | Before | After |
|:--|--:|--:|
| distinct score values | 9 | **35** |
| projects at exactly 100 | 23 | **0** |
| projects at exactly the gate | 37 | **15** |
| median | 80 | 64 |

The gate stays at 40, and a single acquisition modality still scores exactly 40
— so this release is not a change of inclusion policy wearing a change of
arithmetic. Three projects of 117 cross below the line, all from 40 to 36, each
having tripped exactly one weak rule.

## Integer arithmetic, on purpose

Every value is an integer in parts per million. No floating point appears
anywhere.

This is not thrift. The same score must come out of the scanner on a Linux
runner and out of the same code compiled to WebAssembly in a reader's browser,
and *identical* is a stronger word than *close*. A score a reader can recompute
is a score they can dispute — which is the entire point of publishing evidence
beside it.

```bash
cargo rustc --release --target wasm32-unknown-unknown --crate-type cdylib
```

The invariant `Π(1 − wᵢ) > 0` is true of real numbers for free and false of
integers after enough factors, so the implementation floors the product at one
part per million. A test asserts the property of the code rather than of the
mathematics it approximates.

## Three things the score can now answer

```rust
let s = score(&evidence);                      // the number, the decision, the tier
attribute(&evidence, &mut out);                // what each signal actually added
let would_be = score_with(&evidence, addition); // what would raise it
```

Attribution is computed **by removal** — the score with a signal minus the score
without it — because under a saturating combiner a contribution is not a
constant: the same rule adds more to a sparse project than to a well-evidenced
one. `score_with` answers the question a maintainer actually has, which is not
"what is my score" but "what would move it".

## Conformance vectors

Twenty real projects from the live corpus, with their evidence exactly as the
scanner emitted it and their scores pinned. A change that moves any of them has
to be argued for in a diff rather than discovered later in a chart.

## Where the line falls

Python matches terms; this crate combines them. The split is deliberate: a
keyword table wants to be edited in five minutes, and a scoring rule wants to be
identical on every machine for years. Neither language is good at both jobs.

## Constraints

`#![no_std]` · `#![forbid(unsafe_code)]` · `#![deny(missing_docs)]` · zero
dependencies · no allocation · no floating point · rustfmt clean.

## Licensing

Apache-2.0 OR MIT, matching the AxonOS core.

---

<div align="center">

**© The AxonOS Project / Denis Yermakou**

[axonos.org](https://axonos.org) · connect@axonos.org

</div>
