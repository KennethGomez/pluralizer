# pluralizer

Rust package to pluralize or singularize any word based on a count inspired on pluralize NPM package

# Getting Started

[pluralizer.rs is available on crates.io](https://crates.io/crates/pluralizer).
It is recommended to look there for the newest released version, as well as links to the newest builds of the docs.

At the point of the last update of this README, the latest published version could be used like this:

Add the following dependency to your Cargo manifest...

```toml
[dependencies]
pluralizer = "0.3.1"
```

...and see the [docs](https://docs.rs/pluralizer) for how to use it.

# Example

```rust
use pluralizer::pluralize;

fn main() {
    pluralizer::initialize();

    // It can convert to plural
    println!("{}", pluralizer::pluralize("House", 2, true)); // 2 Houses

    // But also can convert to singular
    println!("{}", pluralizer::pluralize("Houses", 1, true)); // 1 House

    // And keep singularization if needed
    println!("{}", pluralizer::pluralize("House", 1, false)); // House

    // Or keep pluralization
    println!("{}", pluralizer::pluralize("Houses", 2, false)); // Houses
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