//! # hexstring
//!
//! The `hexstring` crate provide a convenient hexadecimal string wrapper.
//! It allows all the common conversion expected from a hexadecimal string :
//! - Contains a structured representation of uppercase or lowercase hexadecimal string
//! - Construct from both string and string literal
//! - Convert from and into array of bytes
//!
//! The [`HexString`](crate::HexString) type is a tiny immutable wrapper around string and insure it
//! always contains a valid hexadecimal string.
//!
//! ## Feature flags
//!
//! The following are a list of [Cargo features][cargo-features] that can be enabled or disabled:
//! - **serde**: Enable [serde][serde] support.
//!
//! [cargo-features]: https://doc.rust-lang.org/stable/cargo/reference/features.html#the-features-section
//! [serde]: https://serde.rs

#![feature(adt_const_params)]
#![allow(incomplete_features)]
#![deny(missing_docs)]

use std::borrow::Cow;
use std::convert::{From, TryFrom};
use std::str;

use derive_more::Display;
use hex::FromHexError;

/// Errors than can occurs during [`HexString`] construction.
///
/// Refers to [`FromHexError`][hex::FromHexError] for more details.
pub type Error = FromHexError;

/// Indicates the case of the hexadecimal string.
#[derive(Debug, PartialEq, Eq)]
pub enum Case {
  /// Indicates a lowercase hexadecimal string.
  Lower,
  /// Indicates a uppercase hexadecimal string.
  Upper,
}

/// Provides a structured representation of a hexadecimal string.
///
/// It is guaranteed to be a valid hexadecimal string, whether initialized from a string
/// or from bytes.
/// A valid ['HexString`] should contain only alphanumerical characters such as :
/// - ff04ad992c
/// - FF04AD99C
///
/// And must not mix upper and lower alphabetic characters.
///
/// # Examples
///
/// The idiomatic way to construct a [`HexString`] is to call [`HexString::new`] method with a
/// string.
///
/// ```
/// use hexstring::{HexString, Case};
///
/// let hex = HexString::<{ Case::Upper }>::new("ABCDEF").unwrap();
/// ```
///
/// As the example shown, creating a hexadecimal string is a bit convoluted due to the usage of
/// const generic parameter.
/// Two convenient type aliases must be used instead of the raw [`HexString`] type :
///
/// ```
/// use hexstring::{UpperHexString, LowerHexString};
///
/// let lowercase_hex = LowerHexString::new("abcdef").unwrap();
/// let uppercase_hex = UpperHexString::new("ABCDEF").unwrap();
/// ```
///
/// [`HexString`] has support for conversion from and into array of bytes.
///
/// ```
/// use hexstring::LowerHexString;
///
/// let expected_bytes = [41, 24, 42];
/// let hex = LowerHexString::from(expected_bytes);
/// let bytes = Vec::from(hex);
///
/// assert_eq!(expected_bytes, &bytes[..]);
/// ```
#[cfg_attr(
  feature = "serde",
  derive(serde::Deserialize, serde::Serialize),
  serde(try_from = "String")
)]
#[derive(Display, Default, Clone, Debug, PartialEq, Eq)]
#[display(fmt = "{}", &self.0)]
#[repr(transparent)]
pub struct HexString<const C: Case>(Cow<'static, str>);

/// Convenient alias type to represent uppercase hexadecimal string.
pub type UpperHexString = HexString<{ Case::Upper }>;

/// Convenient alias type to represent lowercase hexadecimal string.
pub type LowerHexString = HexString<{ Case::Lower }>;

impl<const C: Case> HexString<C> {
  /// Constructs a new [`HexString`] from a string.
  ///
  /// # Errors
  /// This method fails if the given string is not a valid hexadecimal.
  pub fn new<S: Into<Cow<'static, str>>>(s: S) -> Result<Self, Error> {
    let s = s.into();

    if s.len() & 1 != 0 {
      return Err(Error::OddLength);
    }

    if let Some((index, c)) = s.chars().enumerate().find(|(_, c)| match C {
      Case::Lower => !matches!(c, '0'..='9' | 'a'..='f'),
      Case::Upper => !matches!(c, '0'..='9' | 'A'..='F'),
    }) {
      return Err(Error::InvalidHexCharacter { c, index });
    }

    Ok(Self(s))
  }

  /// Creates a new [`HexString`] without checking the string.
  ///
  /// # Safety
  /// The string should be a valid hexadecimal string.
  pub unsafe fn new_unchecked<S: Into<Cow<'static, str>>>(s: S) -> Self {
    Self(s.into())
  }
}

impl LowerHexString {
  /// Constructs an [`UpperHexString`] from a [`LowerHexString`].
  ///
  /// This method performs a copy if the internal string is a string literal.
  pub fn to_uppercase(self) -> UpperHexString {
    let mut s = self.0.into_owned();

    s.make_ascii_uppercase();

    unsafe { UpperHexString::new_unchecked(s) }
  }
}

impl UpperHexString {
  /// Constructs a [`LowerHexString`] from an [`UpperHexString`].
  ///
  /// This method performs a copy if the internal string is a string literal.
  pub fn to_lowercase(self) -> LowerHexString {
    let mut s = self.0.into_owned();

    s.make_ascii_lowercase();

    unsafe { LowerHexString::new_unchecked(s) }
  }
}

impl<const C: Case> From<&[u8]> for HexString<C> {
  fn from(bytes: &[u8]) -> Self {
    let s = match C {
      Case::Upper => hex::encode_upper(bytes),
      Case::Lower => hex::encode(bytes),
    };

    unsafe { Self::new_unchecked(s) }
  }
}

impl<const C: Case> From<Vec<u8>> for HexString<C> {
  fn from(bytes: Vec<u8>) -> Self {
    Self::from(&bytes[..])
  }
}

impl<const C: Case, const N: usize> From<[u8; N]> for HexString<C> {
  fn from(bytes: [u8; N]) -> Self {
    Self::from(&bytes[..])
  }
}

impl<const C: Case> From<HexString<C>> for Vec<u8> {
  fn from(s: HexString<C>) -> Self {
    // since `HexString` always represents a valid hexadecimal string, the result of `hex::decode`
    // can be safely unwrapped.
    //
    // Note that this call may panic if the `HexString` has been constructed from `new_unchecked` method.
    hex::decode(s.0.as_ref()).unwrap()
  }
}

impl<const C: Case, const N: usize> TryFrom<HexString<C>> for [u8; N] {
  type Error = Error;

  fn try_from(s: HexString<C>) -> Result<Self, Self::Error> {
    let mut bytes = [0u8; N];

    hex::decode_to_slice(s.0.as_ref(), &mut bytes).map(|_| bytes)
  }
}

// Hide `std::convert::TryFrom` conversion implementation from string used only by
// `serde::Deserialize` mechanism.
//
// It constraints user to use [`HexString::new`] to construct a hexadecimal string.
#[cfg(feature = "serde")]
mod seal {
  use super::*;
  use std::convert::TryFrom;

  #[doc(hidden)]
  impl<const C: Case> TryFrom<String> for HexString<C> {
    type Error = Error;

    fn try_from(s: String) -> Result<Self, Self::Error> {
      Self::new(s)
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn it_constructs_from_owned_str() {
    assert_eq!(
      LowerHexString::new("ab04ff".to_string()),
      Ok(HexString(Cow::Owned("ab04ff".to_string())))
    );
    assert_eq!(
      UpperHexString::new("AB04FF".to_string()),
      Ok(HexString(Cow::Owned("AB04FF".to_string())))
    );
  }

  #[test]
  fn it_constructs_from_borrowed_str() {
    assert_eq!(
      LowerHexString::new("ab04ff"),
      Ok(HexString(Cow::Borrowed("ab04ff")))
    );
    assert_eq!(
      UpperHexString::new("AB04FF"),
      Ok(HexString(Cow::Borrowed("AB04FF")))
    );
  }

  #[test]
  fn it_constructs_from_empty_str() {
    assert!(LowerHexString::new("").is_ok());
    assert!(UpperHexString::new("").is_ok());
  }

  #[test]
  fn it_constructs_from_bytes() {
    assert_eq!(
      LowerHexString::from([42, 15, 5]),
      HexString::<{ Case::Lower }>(Cow::Borrowed("2a0f05"))
    );
    assert_eq!(
      UpperHexString::from([42, 15, 5]),
      HexString::<{ Case::Upper }>(Cow::Borrowed("2A0F05"))
    );
    assert_eq!(
      LowerHexString::from(vec![1, 2, 3, 4, 5]),
      HexString::<{ Case::Lower }>(Cow::Borrowed("0102030405"))
    );
    assert_eq!(
      UpperHexString::from(vec![1, 2, 3, 4, 5]),
      HexString::<{ Case::Upper }>(Cow::Borrowed("0102030405"))
    );
  }

  #[test]
  fn it_rejects_str_with_odd_length() {
    assert_eq!(LowerHexString::new("abc"), Err(Error::OddLength));
    assert_eq!(UpperHexString::new("abcde"), Err(Error::OddLength));
  }

  #[test]
  fn it_rejects_str_with_invalid_chars() {
    assert_eq!(
      LowerHexString::new("abcdZ109"),
      Err(Error::InvalidHexCharacter { c: 'Z', index: 4 })
    );
    assert_eq!(
      UpperHexString::new("ABVCD109"),
      Err(Error::InvalidHexCharacter { c: 'V', index: 2 })
    );
  }

  #[test]
  fn it_constructs_from_unchecked_str() {
    let hex = unsafe { LowerHexString::new_unchecked("0a0b0c0d0e") };
    let bytes = Vec::from(hex);

    assert_eq!(&bytes[..], [10, 11, 12, 13, 14]);
  }

  #[test]
  #[should_panic]
  fn it_fails_to_convert_into_bytes_from_invalid_unchecked_str() {
    let hex = unsafe { LowerHexString::new_unchecked("thisisnotvalid") };
    let _ = Vec::from(hex);
  }

  #[test]
  fn it_converts_into_bytes() {
    let hex = LowerHexString::new("2a1a02").unwrap();
    let bytes = Vec::from(hex);

    assert_eq!(&bytes[..], [42, 26, 2]);

    let hex = UpperHexString::new("2A1A02").unwrap();
    let bytes = Vec::from(hex);

    assert_eq!(&bytes[..], [42, 26, 2]);
  }

  #[test]
  fn it_converts_into_fixed_array_of_bytes() {
    use std::convert::TryInto;

    let bytes: [u8; 4] = LowerHexString::new("142a020a").unwrap().try_into().unwrap();

    assert_eq!(bytes, [20, 42, 2, 10]);

    let bytes: [u8; 5] = UpperHexString::new("142A020A0F")
      .unwrap()
      .try_into()
      .unwrap();

    assert_eq!(bytes, [20, 42, 2, 10, 15]);
  }

  #[test]
  fn it_creates_upper_hex_str_from_lower_hex_str() {
    let s = "aabbccddee";
    let hex = LowerHexString::new(s).unwrap().to_uppercase();
    let expected_hex = HexString::<{ Case::Upper }>(Cow::Owned("AABBCCDDEE".to_string()));

    assert_ne!(s, hex.0.as_ref());
    assert_eq!(hex, expected_hex);

    let hex = LowerHexString::new(s.to_string()).unwrap().to_uppercase();

    assert_eq!(hex, expected_hex);
  }

  #[test]
  fn it_creates_lower_hex_str_from_upper_str() {
    let s = "AABBCCDDEE";
    let hex = UpperHexString::new(s).unwrap().to_lowercase();
    let expected_hex = HexString::<{ Case::Lower }>(Cow::Owned("aabbccddee".to_string()));

    assert_ne!(s, hex.0.as_ref());
    assert_eq!(hex, expected_hex);

    let hex = UpperHexString::new(s.to_string()).unwrap().to_lowercase();

    assert_eq!(hex, expected_hex);
  }

  #[cfg(feature = "serde")]
  mod serde {
    use super::*;
    use serde_json::error::Category;

    #[test]
    fn it_deser_hex_str() {
      let result: Result<LowerHexString, _> = serde_json::from_str("\"abcd09\"");

      assert!(result.is_ok());

      let result: Result<UpperHexString, _> = serde_json::from_str("\"ABCD09\"");

      assert!(result.is_ok());
    }

    #[test]
    fn it_fails_to_deser_invalid_hex_str() {
      let result: Result<LowerHexString, serde_json::Error> =
        serde_json::from_str("\"invalid hex str\"");

      assert_eq!(result.unwrap_err().classify(), Category::Data);

      let result: Result<UpperHexString, serde_json::Error> =
        serde_json::from_str("\"INVALID HEX STR\"");

      assert_eq!(result.unwrap_err().classify(), Category::Data);
    }
  }
}
