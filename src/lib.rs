//! # hexstring
//!
//! The `hexstring` crate provide a convenient hexadecimal string wrapper.
//! It allows all the common conversion expected from a hexadecimal string :
//! - Contains a structured representation of uppercase or lowercase hexadecimal string
//! - Construct from both string and string literal
//! - Convert from and into array of bytes
//!
//! The [`HexString`] type is a tiny immutable wrapper around string and insure it
//! always contains a valid hexadecimal string.
//!
//! ## Feature flags
//!
//! The following are a list of [Cargo features][cargo-features] that can be enabled or disabled:
//! - **serde**: Enable [serde][serde] support.
//!
//! [cargo-features]: https://doc.rust-lang.org/stable/cargo/reference/features.html#the-features-section
//! [serde]: https://serde.rs

#![deny(missing_docs)]

use std::borrow::Cow;
use std::convert::{From, TryFrom};
use std::marker::PhantomData;
use std::str::{self, FromStr};

use derive_more::Display;
use hex::FromHexError;

/// Errors than can occurs during [`HexString`] construction.
///
/// Refers to [`FromHexError`] for more details.
pub type Error = FromHexError;

/// Convenient alias type to represent uppercase hexadecimal string.
pub type UpperHexString = HexString<Uppercase>;

/// Convenient alias type to represent lowercase hexadecimal string.
pub type LowerHexString = HexString<Lowercase>;

mod private {
  /// A sealed trait to prevent external implementations of [`Case`].
  pub trait Sealed {}
}

/// Provides encoding and validation for hexadecimal strings according to the case.
///
/// This trait is sealed to prevent external implementations, ensuring that only `Lowercase` and
/// `Uppercase` can be used as cases.
pub trait Case: private::Sealed {
  /// Encodes the given bytes into a hexadecimal string according to the case.
  fn encode(bytes: &[u8]) -> String;
  /// Checks if the given character is a valid hexadecimal character according to the case.
  fn is_valid(c: char) -> bool;
}

/// Lowercase hexadecimal string representation.
#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Lowercase;

impl Case for Lowercase {
  fn encode(bytes: &[u8]) -> String {
    hex::encode(bytes)
  }

  fn is_valid(c: char) -> bool {
    matches!(c, '0'..='9' | 'a'..='f')
  }
}

impl private::Sealed for Lowercase {}

/// Uppercase hexadecimal string representation.
#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Uppercase;

impl Case for Uppercase {
  fn encode(bytes: &[u8]) -> String {
    hex::encode_upper(bytes)
  }

  fn is_valid(c: char) -> bool {
    matches!(c, '0'..='9' | 'A'..='F')
  }
}

impl private::Sealed for Uppercase {}


/// Provides a structured representation of a hexadecimal string.
///
/// It is guaranteed to be a valid hexadecimal string, whether initialized from a string
/// or from bytes.
/// A valid [`HexString`] should contain only alphanumerical characters such as :
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
/// use hexstring::{HexString, Uppercase};
///
/// let hex = HexString::<Uppercase>::new("ABCDEF").unwrap();
/// ```
///
/// As the example shown, creating a hexadecimal string is a bit convoluted due to the usage of
/// generic type.
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
#[derive(Clone, Debug, Default, Display, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[display(fmt = "{}", _0)]
#[repr(transparent)]
pub struct HexString<C: Case>(Cow<'static, str>, PhantomData<C>);

impl<C: Case> HexString<C> {
  /// Constructs a new [`HexString`] from a string.
  ///
  /// # Errors
  /// This method fails if the given string is not a valid hexadecimal, i.e. if it has an odd length
  /// or contains invalid characters.
  pub fn new(s: impl Into<Cow<'static, str>>) -> Result<Self, Error> {
    let s = s.into();

    if s.len() & 1 != 0 {
      return Err(Error::OddLength);
    }

    if let Some((index, c)) = s.chars().enumerate().find(|(_, c)| !C::is_valid(*c)) {
      return Err(Error::InvalidHexCharacter { c, index });
    }

    Ok(Self(s, PhantomData))
  }

  /// Creates a new [`HexString`] without checking the string.
  ///
  /// # Safety
  /// The string should be a valid hexadecimal string.
  pub unsafe fn new_unchecked(s: impl Into<Cow<'static, str>>) -> Self {
    Self(s.into(), PhantomData)
  }
}

impl HexString<Lowercase> {
  /// Constructs an [`HexString<Uppercase>`] from a [`HexString<Lowercase>`].
  ///
  /// This method performs a copy if the internal string is a string literal.
  pub fn to_uppercase(self) -> HexString<Uppercase> {
    let mut s = self.0.into_owned();

    s.make_ascii_uppercase();

    unsafe { HexString::new_unchecked(s) }
  }
}

impl UpperHexString {
  /// Constructs a [`HexString<Lowercase>`] from an [`HexString<Uppercase>`].
  ///
  /// This method performs a copy if the internal string is a string literal.
  pub fn to_lowercase(self) -> HexString<Lowercase> {
    let mut s = self.0.into_owned();

    s.make_ascii_lowercase();

    unsafe { HexString::new_unchecked(s) }
  }
}

impl<C: Case> From<&[u8]> for HexString<C> {
  fn from(bytes: &[u8]) -> Self {
    let s = C::encode(bytes);

    unsafe { Self::new_unchecked(s) }
  }
}

impl<C: Case> From<Vec<u8>> for HexString<C> {
  fn from(bytes: Vec<u8>) -> Self {
    Self::from(&bytes[..])
  }
}

impl<C: Case, const N: usize> From<[u8; N]> for HexString<C> {
  fn from(bytes: [u8; N]) -> Self {
    Self::from(&bytes[..])
  }
}

impl<C: Case> From<HexString<C>> for Vec<u8> {
  fn from(s: HexString<C>) -> Self {
    // since `HexString` always represents a valid hexadecimal string, the result of `hex::decode`
    // can be safely unwrapped.
    //
    // Note that this call may panic if the `HexString` has been constructed from `new_unchecked`
    // method.
    hex::decode(s.0.as_ref()).unwrap()
  }
}

impl<C: Case> FromStr for HexString<C> {
  type Err = Error;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    Self::new(s.to_owned())
  }
}

impl<C: Case, const N: usize> TryFrom<HexString<C>> for [u8; N] {
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
  impl<C: Case> TryFrom<String> for HexString<C> {
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
      Ok(HexString(Cow::Owned("ab04ff".to_string()), PhantomData))
    );
    assert_eq!(
      UpperHexString::new("AB04FF".to_string()),
      Ok(HexString(Cow::Owned("AB04FF".to_string()), PhantomData))
    );
  }

  #[test]
  fn it_constructs_from_borrowed_str() {
    assert_eq!(
      LowerHexString::new("ab04ff"),
      Ok(HexString(Cow::Borrowed("ab04ff"), PhantomData))
    );
    assert_eq!(
      UpperHexString::new("AB04FF"),
      Ok(HexString(Cow::Borrowed("AB04FF"), PhantomData))
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
      HexString::<Lowercase>(Cow::Borrowed("2a0f05"), PhantomData)
    );
    assert_eq!(
      UpperHexString::from([42, 15, 5]),
      HexString::<Uppercase>(Cow::Borrowed("2A0F05"), PhantomData)
    );
    assert_eq!(
      LowerHexString::from(vec![1, 2, 3, 4, 5]),
      HexString::<Lowercase>(Cow::Borrowed("0102030405"), PhantomData)
    );
    assert_eq!(
      UpperHexString::from(vec![1, 2, 3, 4, 5]),
      HexString::<Uppercase>(Cow::Borrowed("0102030405"), PhantomData)
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
    let expected_hex = HexString::<Uppercase>(Cow::Owned("AABBCCDDEE".to_string()), PhantomData);

    assert_ne!(s, hex.0.as_ref());
    assert_eq!(hex, expected_hex);

    let hex = LowerHexString::new(s.to_string()).unwrap().to_uppercase();

    assert_eq!(hex, expected_hex);
  }

  #[test]
  fn it_creates_lower_hex_str_from_upper_str() {
    let s = "AABBCCDDEE";
    let hex = UpperHexString::new(s).unwrap().to_lowercase();
    let expected_hex = HexString::<Lowercase>(Cow::Owned("aabbccddee".to_string()), PhantomData);

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
