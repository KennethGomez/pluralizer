# pluralizer

Rust package to pluralize or singularize any word based on a count inspired on pluralize NPM package.

It will keep plurals are plurals if the count given is not 1, either way, it is going to keep the  singular form if the count given is 1

[![Rust](https://github.com/KennethGomez/pluralizer/actions/workflows/rust.yml/badge.svg)](https://github.com/KennethGomez/pluralizer/actions/workflows/rust.yml)
[![Latest version](https://img.shields.io/crates/v/pluralizer.svg)](https://crates.io/crates/pluralizer)
[![Downloads](https://img.shields.io/crates/d/pluralizer)](https://crates.io/crates/pluralizer)
[![Documentation](https://docs.rs/pluralizer/badge.svg)](https://docs.rs/pluralizer)
[![License](https://img.shields.io/crates/l/pluralizer.svg)](https://github.com/KennethGomez/pluralizer#license)

## Performance

This library has been benchmarked with [Criterion](https://crates.io/crates/criterion). Below are example results observed in a recent test environment:

| Benchmark                 | Mean    | Notes                                        |
|---------------------------|---------|----------------------------------------------|
| **pluralize**             | ~7.6 ms | Performing repeated pluralization operations |
| **add rules + pluralize** | ~7.8 ms | Adding and applying custom rules             |

> Times can vary depending on factors like your CPU, Rust compiler version, and benchmark configuration.

# Getting Started

[pluralizer.rs is available on crates.io](https://crates.io/crates/pluralizer).
It is recommended to look there for the newest released version, as well as links to the newest builds of the docs.

At the point of the last update of this README, the latest published version could be used like this:

Add the following dependency to your Cargo manifest...

```toml
[dependencies]
pluralizer = "0.4.0"
```

...and see the [docs](https://docs.rs/pluralizer) for how to use it.

# Example

```rust
use pluralizer::pluralize;

fn main() {
    // It can convert to plural
    println!("{}", pluralize("House", 2, true)); // 2 Houses

    // But also can convert to singular
    println!("{}", pluralize("Houses", 1, true)); // 1 House

    // And keep singularization if needed
    println!("{}", pluralize("House", 1, false)); // House

    // Or keep pluralization
    println!("{}", pluralize("Houses", 2, false)); // Houses
}
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