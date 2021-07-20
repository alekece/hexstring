[![crates.io](https://img.shields.io/crates/v/hexstring.svg)](https://crates.io/crates/hexstring)
[![Documentation](https://docs.rs/hexstring/badge.svg)](https://docs.rs/hexstring)
[![MIT licensed](https://img.shields.io/crates/l/hexstring.svg)](./LICENSE)

<!-- cargo-sync-readme start -->

# hexstring

The `hexstring` crate provide a convenient hexadecimal string wrapper.
It allows all the common conversion expected from a hexadecimal string :
- Contains a structured representation of uppercase or lowercase hexadecimal string
- Construct from both string and string literal
- Convert from and into array of bytes

The [`HexString`](https://docs.rs/hexstring/latest/hexstring/struct.HexString.html) type is a tiny immutable wrapper around string and insure it always contains a
valid hexadecimal string.

## Feature flags

The following are a list of [Cargo features][cargo-features] that can be enabled or disabled:
- **serde**: Enable [serde][serde] support.

[cargo-features]: https://doc.rust-lang.org/stable/cargo/reference/features.html#the-features-section
[serde]: https://serde.rs

<!-- cargo-sync-readme end -->

## Requirements
`hexstring` crate uses unstable constant generic type internally.
To compile the library in any project, build it in nightly mode such as :

``` sh
rustup override set nightly
```

## License

Licensed under MIT license ([LICENSE](LICENSE) or http://opensource.org/licenses/MIT)
