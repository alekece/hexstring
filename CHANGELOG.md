# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.3] - 2022-04-28
### Changed
- Bump Rust edition to 2021

## [0.1.2] - 2021-08-11
### Added
- `new_unchecked` method which skips string checking.

### Fixed
- Update CHANGELOG links

## [0.1.1] - 2021-07-21
### Added
- Convenient conversion to uppercase and lowercase hexadecimal string.

## [0.1.0] - 2021-07-20
### Added
- `HexString` type which is a structured representation of a hexadecimal string.
- Conversion from and into array of bytes.
- Convenient type aliases `UpperHexString` and `LowerHexString`.
- Feature flag `serde` for serde support on `HexString` type.

[Unreleased]: https://github.com/alekece/hextring-rs/compare/v0.1.3...HEAD
[0.1.3]: https://github.com/alekece/hexstring-rs/releases/tag/v0.1.3
[0.1.2]: https://github.com/alekece/hexstring-rs/releases/tag/v0.1.2
[0.1.1]: https://github.com/alekece/hexstring-rs/releases/tag/v0.1.1
[0.1.0]: https://github.com/alekece/hexstring-rs/releases/tag/v0.1.0
