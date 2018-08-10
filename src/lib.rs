// Copyright 2013-2014 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Hex binary-to-text encoding

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "std")]
extern crate core;

#[cfg(feature = "std")]
use std::fmt;

#[cfg(not(feature = "std"))]
use core::fmt;

use core::iter::{self, FromIterator};

pub use self::FromHexError::*;
/// A trait for converting a value to hexadecimal encoding
pub trait ToHex {
    /// Converts the value of `self` to a hex value, constructed from
    /// an iterator of characaters.
    fn to_hex<T: FromIterator<char>>(&self) -> T;
}

static CHARS: &'static [u8] = b"0123456789abcdef";

impl ToHex for [u8] {
    /// Turn a vector of `u8` bytes into a hexadecimal string.
    ///
    /// # Example
    ///
    /// ```rust
    /// extern crate rustc_hex;
    /// use rustc_hex::ToHex;
    ///
    /// fn main () {
    ///     let str: String = [52,32].to_hex();
    ///     println!("{}", str);
    /// }
    /// ```
    fn to_hex<T: FromIterator<char>>(&self) -> T {
        struct SliceToHex<'a> {
            live: Option<char>,
            inner: ::core::slice::Iter<'a, u8>,
        }

        impl<'a> Iterator for SliceToHex<'a> {
            type Item = char;

            fn next(&mut self) -> Option<char> {
                if let Some(live) = self.live.take() {
                    return Some(live);
                }

                self.inner.next().map(|&byte| {
                    let current = CHARS[(byte >> 4) as usize] as char;
                    self.live = Some(CHARS[(byte & 0xf) as usize] as char);
                    current
                })
            }

            fn size_hint(&self) -> (usize, Option<usize>) {
                let len = self.len();
                (len, Some(len))
            }
        }

        impl<'a> iter::ExactSizeIterator for SliceToHex<'a> {
            fn len(&self) -> usize {
                let mut len = self.inner.len() * 2;
                if self.live.is_some() {
                    len += 1;
                }
                len
            }
        }

        SliceToHex {
            live: None,
            inner: self.iter()
        }.collect()
    }
}

impl<'a, T: ?Sized + ToHex> ToHex for &'a T {
    fn to_hex<U: FromIterator<char>>(&self) -> U {
        (**self).to_hex()
    }
}

/// A trait for converting hexadecimal encoded values
pub trait FromHex {
    /// Converts the value of `self`, interpreted as hexadecimal encoded data,
    /// into an owned value constructed from an iterator of bytes.
    fn from_hex<T: FromIterator<u8>>(&self) -> Result<T, FromHexError>;
}

/// Errors that can occur when decoding a hex encoded string
#[derive(Clone, Copy)]
pub enum FromHexError {
    /// The input contained a character not part of the hex format
    InvalidHexCharacter(char, usize),
    /// The input had an invalid length
    InvalidHexLength,
}

#[cfg(feature = "std")]
impl fmt::Debug for FromHexError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            InvalidHexCharacter(ch, idx) =>
                write!(f, "Invalid character '{}' at position {}", ch, idx),
            InvalidHexLength => write!(f, "Invalid input length"),
        }
    }
}

#[cfg(not(feature = "std"))]
impl fmt::Debug for FromHexError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            InvalidHexCharacter(ch, idx) =>
                write!(f, "Invalid character '{}' at position {}", ch, idx),
            InvalidHexLength => write!(f, "Invalid input length"),
        }
    }
}

#[cfg(feature = "std")]
impl ::std::error::Error for FromHexError {
    fn description(&self) -> &str {
        match *self {
            InvalidHexCharacter(_, _) => "invalid character",
            InvalidHexLength => "invalid length",
        }
    }
}

#[cfg(feature = "std")]
impl fmt::Display for FromHexError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(&self, f)
    }
}

#[cfg(not(feature = "std"))]
impl fmt::Display for FromHexError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            InvalidHexCharacter(ch, idx) => {
                f.write_str("invalid character: ")?;
                ch.fmt(f)?;
                f.write_str(" at index ")?;
                idx.fmt(f)
            }
            InvalidHexLength => f.write_str("invalid length"),
        }
    }
}

impl FromHex for str {
    /// Convert any hexadecimal encoded string (literal, `@`, `&`, or `~`)
    /// to the byte values it encodes.
    ///
    /// You can use the `String::from_utf8` function to turn a
    /// `Vec<u8>` into a string with characters corresponding to those values.
    ///
    /// # Example
    ///
    /// This converts a string literal to hexadecimal and back.
    ///
    /// ```rust
    /// extern crate rustc_hex;
    /// use rustc_hex::{FromHex, ToHex};
    ///
    /// fn main () {
    ///     let hello_str: String = "Hello, World".as_bytes().to_hex();
    ///     println!("{}", hello_str);
    ///     let bytes: Vec<u8> = hello_str.from_hex().unwrap();
    ///     println!("{:?}", bytes);
    ///     let result_str = String::from_utf8(bytes).unwrap();
    ///     println!("{}", result_str);
    /// }
    /// ```
    fn from_hex<T: FromIterator<u8>>(&self) -> Result<T, FromHexError> {
        struct StrFromHex<'a> {
            err: &'a mut Result<(), FromHexError>,
            inner: &'a str,
            iter: iter::Enumerate<::core::str::Bytes<'a>>,
        }

        impl<'a> Iterator for StrFromHex<'a> {
            type Item = u8;

            fn next(&mut self) -> Option<u8> {
                let mut modulus = 0;
                let mut buf = 0;
                for (idx, byte) in &mut self.iter {
                    buf <<= 4;

                    match byte {
                        b'A'...b'F' => buf |= byte - b'A' + 10,
                        b'a'...b'f' => buf |= byte - b'a' + 10,
                        b'0'...b'9' => buf |= byte - b'0',
                        b' '|b'\r'|b'\n'|b'\t' => {
                            buf >>= 4;
                            continue
                        }
                        _ => {
                            let ch = self.inner[idx..].chars().next().unwrap();
                            *self.err = Err(InvalidHexCharacter(ch, idx));
                            return None;
                        }
                    }

                    modulus += 1;
                    if modulus == 2 {
                        return Some(buf);
                    }
                }

                if modulus != 0 {
                    *self.err = Err(InvalidHexLength);
                }

                None
            }
        }

        let mut err = Ok(());
        let val: T = StrFromHex {
            err: &mut err,
            inner: self,
            iter: self.bytes().enumerate(),
        }.collect();

        err.map(move |_| val)
    }
}

impl<'a, T: ?Sized + FromHex> FromHex for &'a T {
    fn from_hex<U: FromIterator<u8>>(&self) -> Result<U, FromHexError> {
        (**self).from_hex()
    }
}

#[cfg(test)]
mod tests {
    use super::{FromHex, ToHex};

    #[test]
    pub fn test_to_hex() {
        assert_eq!("foobar".as_bytes().to_hex::<String>(), "666f6f626172");
    }

    #[test]
    pub fn test_from_hex_okay() {
        assert_eq!("666f6f626172".from_hex::<Vec<_>>().unwrap(),
                   b"foobar");
        assert_eq!("666F6F626172".from_hex::<Vec<_>>().unwrap(),
                   b"foobar");
    }

    #[test]
    pub fn test_from_hex_odd_len() {
        assert!("666".from_hex::<Vec<_>>().is_err());
        assert!("66 6".from_hex::<Vec<_>>().is_err());
    }

    #[test]
    pub fn test_from_hex_invalid_char() {
        assert!("66y6".from_hex::<Vec<_>>().is_err());
    }

    #[test]
    pub fn test_from_hex_ignores_whitespace() {
        assert_eq!("666f 6f6\r\n26172 ".from_hex::<Vec<_>>().unwrap(),
                   b"foobar");
    }

    #[test]
    pub fn test_to_hex_all_bytes() {
        for i in 0..256 {
            assert_eq!([i as u8].to_hex::<String>(), format!("{:02x}", i));
        }
    }

    #[test]
    pub fn test_from_hex_all_bytes() {
        for i in 0..256 {
            let ii: &[u8] = &[i as u8];
            assert_eq!(format!("{:02x}", i).from_hex::<Vec<_>>().unwrap(),
                       ii);
            assert_eq!(format!("{:02X}", i).from_hex::<Vec<_>>().unwrap(),
                       ii);
        }
    }
}
