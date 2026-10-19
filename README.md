# pluralizer

Rust package to pluralize or singularize any word based on a count, inspired by the [pluralize](https://www.npmjs.com/package/pluralize) NPM package.

A count of exactly 1 produces the singular form and any other count produces the plural form. Words that are already in the requested form are kept as they are.

[![Rust](https://github.com/KennethGomez/pluralizer/actions/workflows/rust.yml/badge.svg)](https://github.com/KennethGomez/pluralizer/actions/workflows/rust.yml)
[![Latest version](https://img.shields.io/crates/v/pluralizer.svg)](https://crates.io/crates/pluralizer)
[![Downloads](https://img.shields.io/crates/d/pluralizer)](https://crates.io/crates/pluralizer)
[![Documentation](https://docs.rs/pluralizer/badge.svg)](https://docs.rs/pluralizer)
[![License](https://img.shields.io/crates/l/pluralizer.svg)](https://github.com/KennethGomez/pluralizer#license)

## Performance

This library is benchmarked with [Criterion](https://crates.io/crates/criterion) (`cargo bench`). Example results:

| Benchmark                    | Time     | Notes                                                                    |
|------------------------------|----------|--------------------------------------------------------------------------|
| `pluralize 10k calls`        | ~1.5 ms  | 10,000 calls pluralizing `"house"`, ~150 ns per call                     |
| `corpus/plural`              | ~46 µs   | 213 mixed words (regular, irregular, uncountable, non-ASCII; lower, Title and UPPER case) |
| `corpus/singular`            | ~49 µs   | The same words, singularized                                             |
| `custom rules/pluralize/100` | ~300 ns  | One call with 100 custom plural, irregular and uncountable rules added   |

Measured on an Apple M4 Pro with Rust 1.87.

> Times vary with your CPU, Rust compiler version and benchmark configuration.

# Getting Started

[pluralizer.rs is available on crates.io](https://crates.io/crates/pluralizer).
It is recommended to look there for the newest released version, as well as links to the newest builds of the docs.

Add the following dependency to your Cargo manifest...

```toml
[dependencies]
pluralizer = "0.6"
```

...and see the [docs](https://docs.rs/pluralizer) for how to use it.

# Example

```rust
use pluralizer::pluralize;

// It can convert to plural
assert_eq!(pluralize("House", 2, true), "2 Houses");

// But also can convert to singular
assert_eq!(pluralize("Houses", 1, true), "1 House");

// And keep singularization if needed
assert_eq!(pluralize("House", 1, false), "House");

// Or keep pluralization
assert_eq!(pluralize("Houses", 2, false), "Houses");
```

Rules can also be added at runtime. They are global, so they apply to every later call on every thread:

```rust
use pluralizer::{add_irregular_rule, add_uncountable_rule, pluralize, UncountableRule};

add_irregular_rule("octopus".to_string(), "octopodes".to_string());
add_uncountable_rule(UncountableRule::String("paper".to_string()));

assert_eq!(pluralize("octopus", 2, false), "octopodes");
assert_eq!(pluralize("paper", 2, false), "paper");
```

## License

Licensed under either of

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE](LICENSE) or http://opensource.org/licenses/MIT)

at your option

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.
