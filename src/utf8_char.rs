/* Copyright 2016 Torbjørn Birch Moltu
 *
 * Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
 * http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
 * http://opensource.org/licenses/MIT>, at your option. This file may not be
 * copied, modified, or distributed except according to those terms.
 */

pub use errors::FromStrError;
use error::{InvalidUtf8Slice,InvalidUtf8Array};
use Utf8Iterator;
use CharExt;
use U8UtfExt;
use Utf16Char;
extern crate core;
use self::core::{hash, fmt, str, ptr};
use self::core::borrow::Borrow;
use self::core::ops::Deref;
use self::core::mem::transmute;
#[cfg(feature="std")]
use self::core::iter::FromIterator;
#[cfg(feature="std")]
#[allow(deprecated)]
use std::ascii::AsciiExt;
#[cfg(feature="ascii")]
extern crate ascii;
#[cfg(feature="ascii")]
use self::ascii::{AsciiChar,ToAsciiChar,ToAsciiCharError};


// I don't think there is any good default value for char, but char does.
#[derive(Default)]
// char doesn't do anything more advanced than u32 for Eq/Ord, so we shouldn't either.
// The default impl of Ord for arrays works out because longer codepoints
//     start with more ones, so if they're equal, the length is the same,
// breaks down for values above 0x1f_ff_ff but those can only be created by unsafe code.
#[derive(PartialEq,Eq, PartialOrd,Ord)]

#[derive(Clone,Copy)]


/// An unicode codepoint stored as UTF-8.
///
/// It can be borrowed as a `str`, and has the same size as `char`.
pub struct Utf8Char {
    bytes: [u8; 4],
}


  /////////////////////
 //conversion traits//
/////////////////////
impl str::FromStr for Utf8Char {
    type Err = FromStrError;
    /// The string must contain exactly one codepoint
    fn from_str(s: &str) -> Result<Self, FromStrError> {
        if s.is_empty() {
            Err(FromStrError::Empty)
        } else if s.len() != 1+s.as_bytes()[0].extra_utf8_bytes_unchecked() {
            Err(FromStrError::MultipleCodepoints)
        } else {
            let mut bytes = [0; 4];
            bytes[..s.len()].copy_from_slice(s.as_bytes());
            Ok(Utf8Char{bytes: bytes})
        }
    }
}
impl From<Utf16Char> for Utf8Char {
    fn from(utf16: Utf16Char) -> Utf8Char {
        match utf16.to_tuple() {
            (a @ 0...0x00_7f, _) => {
                Utf8Char{ bytes: [a as u8, 0, 0, 0] }
            },
            (u @ 0...0x07_ff, _) => {
                let b = 0x80 |  (u & 0x00_3f) as u8;
                let a = 0xc0 | ((u & 0x07_c0) >> 6) as u8;
                Utf8Char{ bytes: [a, b, 0, 0] }
            },
            (u, None) => {
                let c = 0x80 |  (u & 0x00_3f) as u8;
                let b = 0x80 | ((u & 0x0f_c0) >> 6) as u8;
                let a = 0xe0 | ((u & 0xf0_00) >> 12) as u8;
                Utf8Char{ bytes: [a, b, c, 0] }
            },
            (f, Some(s)) => {
                let f = f + (0x01_00_00u32 >> 10) as u16;
                let d = 0x80 |  (s & 0x00_3f) as u8;
                let c = 0x80 | ((s & 0x03_c0) >> 6) as u8
                             | ((f & 0x00_03) << 4) as u8;
                let b = 0x80 | ((f & 0x00_fc) >> 2) as u8;
                let a = 0xf0 | ((f & 0x07_00) >> 8) as u8;
                Utf8Char{ bytes: [a, b, c, d] }
            }
        }
    }
}
impl From<char> for Utf8Char {
    fn from(c: char) -> Self {
        Utf8Char{ bytes: c.to_utf8_array().0 }
    }
}
impl From<Utf8Char> for char {
    fn from(uc: Utf8Char) -> char {
        unsafe{ char::from_utf8_exact_slice_unchecked(&uc.bytes[..uc.len()]) }
    }
}
impl IntoIterator for Utf8Char {
    type Item=u8;
    type IntoIter=Utf8Iterator;
    /// Iterate over the byte values.
    fn into_iter(self) -> Utf8Iterator {
        Utf8Iterator::from(self)
    }
}
#[cfg(feature="std")]
impl Extend<Utf8Char> for Vec<u8> {
    fn extend<I:IntoIterator<Item=Utf8Char>>(&mut self, iter: I) {
        let iter = iter.into_iter();
        self.reserve(iter.size_hint().0);
        for u8c in iter {
            self.extend_from_slice(u8c.as_bytes());
        }
    }
}
#[cfg(feature="std")]
impl Extend<Utf8Char> for String {
    fn extend<I:IntoIterator<Item=Utf8Char>>(&mut self, iter: I) {
        unsafe { self.as_mut_vec().extend(iter) }
    }
}
#[cfg(feature="std")]
impl FromIterator<Utf8Char> for Vec<u8> {
    fn from_iter<I:IntoIterator<Item=Utf8Char>>(iter: I) -> Self {
        let mut vec = Vec::new();
        vec.extend(iter);
        return vec;
    }
}
#[cfg(feature="std")]
impl FromIterator<Utf8Char> for String {
    fn from_iter<I:IntoIterator<Item=Utf8Char>>(iter: I) -> Self {
        let mut string = String::new();
        string.extend(iter);
        return string;
    }
}


  /////////////////
 //getter traits//
/////////////////
impl AsRef<[u8]> for Utf8Char {
    fn as_ref(&self) -> &[u8] {
        &self.bytes[..self.len()]
    }
}
impl AsRef<str> for Utf8Char {
    fn as_ref(&self) -> &str {
        unsafe{ str::from_utf8_unchecked( self.as_ref() ) }
    }
}
impl Borrow<[u8]> for Utf8Char {
    fn borrow(&self) -> &[u8] {
        self.as_ref()
    }
}
impl Borrow<str> for Utf8Char {
    fn borrow(&self) -> &str {
        self.as_ref()
    }
}
impl Deref for Utf8Char {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}


  ////////////////
 //ascii traits//
////////////////
#[cfg(feature="std")]
#[allow(deprecated)]
impl AsciiExt for Utf8Char {
    type Owned = Utf8Char;
    fn is_ascii(&self) -> bool {
        self.bytes[0].is_ascii()
    }
    fn eq_ignore_ascii_case(&self,  other: &Self) -> bool {
        if self.is_ascii() {self.bytes[0].eq_ignore_ascii_case(&other.bytes[0])}
        else               {self == other}
    }
    fn to_ascii_uppercase(&self) -> Self::Owned {
        let mut uc = *self;
        uc.make_ascii_uppercase();
        uc
    }
    fn to_ascii_lowercase(&self) -> Self::Owned {
        let mut uc = *self;
        uc.make_ascii_lowercase();
        uc
    }
    fn make_ascii_uppercase(&mut self) {
        self.bytes[0].make_ascii_uppercase()
    }
    fn make_ascii_lowercase(&mut self) {
        self.bytes[0].make_ascii_lowercase();
    }
}

#[cfg(feature="ascii")]
/// Requires the feature "ascii".
impl From<AsciiChar> for Utf8Char {
    fn from(ac: AsciiChar) -> Self {
        Utf8Char{ bytes: [ac.as_byte(),0,0,0] }
    }
}
#[cfg(feature="ascii")]
/// Requires the feature "ascii".
impl ToAsciiChar for Utf8Char {
    fn to_ascii_char(self) -> Result<AsciiChar, ToAsciiCharError> {
        self.bytes[0].to_ascii_char()
    }
    unsafe fn to_ascii_char_unchecked(self) -> AsciiChar {
        self.bytes[0].to_ascii_char_unchecked()
    }
}


  /////////////////////////////////////////////////////////
 //Genaral traits that cannot be derived to emulate char//
/////////////////////////////////////////////////////////
impl hash::Hash for Utf8Char {
    fn hash<H : hash::Hasher>(&self,  state: &mut H) {
        self.to_char().hash(state);
    }
}
impl fmt::Debug for Utf8Char {
    fn fmt(&self,  fmtr: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(&self.to_char(), fmtr)
    }
}
impl fmt::Display for Utf8Char {
    fn fmt(&self,  fmtr: &mut fmt::Formatter) -> fmt::Result {
        fmtr.write_str(self.as_str())
    }
}


  ///////////////////////////////////////////////////////
 //pub impls that should be together for nicer rustdoc//
///////////////////////////////////////////////////////
impl Utf8Char {
    /// Validate the start of a UTF-8 slice and store it.
    /// Also returns how many bytes were needed.
    ///
    /// If it's a str and you know it contains only one codepoint,
    /// use `.from_str()` to skip the validation.
    pub fn from_slice_start(src: &[u8]) -> Result<(Self,usize),InvalidUtf8Slice> {
        char::from_utf8_slice_start(src).map(|(_,len)| {
            let mut bytes = [0; 4];
            bytes[..len].copy_from_slice(&src[..len]);
            (Utf8Char{bytes: bytes}, len)
        })
    }
    /// A `from_slice_start()` that doesn't validate the codepoint.
    ///
    /// # Safety
    /// The slice must be non-empty and start with a valid UTF-8 codepoint.  
    /// Invalid or incomplete values might cause buffer overflows and overreads.
    pub unsafe fn from_slice_start_unchecked(src: &[u8]) -> (Self,usize) {
        let len = 1+src.get_unchecked(0).extra_utf8_bytes_unchecked();
        let mut bytes = [0; 4];
        ptr::copy_nonoverlapping(src.as_ptr(), &mut bytes[0] as *mut u8, len);
        (Utf8Char{ bytes: bytes }, len)
    }
    /// Validate the array and store it.
    pub fn from_array(utf8: [u8;4]) -> Result<Self,InvalidUtf8Array> {
        unsafe {
            try!(char::from_utf8_array(utf8));
            let extra = utf8[0].extra_utf8_bytes_unchecked() as u32;
            let mask = u32::from_le(0xff_ff_ff_ff >> 8*(3-extra));
            let unused_zeroed = mask  &  transmute::<_,u32>(utf8);
            Ok(Utf8Char{ bytes: transmute(unused_zeroed) })
        }
    }
    /// Unused bytes must be zero
    pub unsafe fn from_array_unchecked(utf8: [u8;4]) -> Self {
        Utf8Char{ bytes: utf8 }
    }

    /// Result is 1...4 and identical to `.as_ref().len()` or
    /// `.as_char().len_utf8()`.
    /// There is no .is_emty() because this type is never empty.
    pub fn len(self) -> usize {
        self.bytes[0].extra_utf8_bytes_unchecked() + 1
    }

    /// Checks that the codepoint is an ASCII character.
    pub fn is_ascii(&self) -> bool {
        self.bytes[0] <= 127
    }
    /// Checks that two characters are an ASCII case-insensitive match.
    ///
    /// Is equivalent to `a.to_ascii_lowercase() == b.to_ascii_lowercase()`.
    #[cfg(feature="std")]
    pub fn eq_ignore_ascii_case(&self,  other: &Self) -> bool {
        if self.is_ascii() {self.bytes[0].eq_ignore_ascii_case(&other.bytes[0])}
        else               {self == other}
    }
    /// Converts the character to its ASCII upper case equivalent.
    ///
    /// ASCII letters 'a' to 'z' are mapped to 'A' to 'Z',
    /// but non-ASCII letters are unchanged.
    #[cfg(feature="std")]
    pub fn to_ascii_uppercase(&self) -> Self {
        let mut uc = *self;
        uc.make_ascii_uppercase();
        uc
    }
    /// Converts the character to its ASCII lower case equivalent.
    ///
    /// ASCII letters 'A' to 'Z' are mapped to 'a' to 'z',
    /// but non-ASCII letters are unchanged.
    #[cfg(feature="std")]
    pub fn to_ascii_lowercase(&self) -> Self {
        let mut uc = *self;
        uc.make_ascii_lowercase();
        uc
    }
    /// Converts the character to its ASCII upper case equivalent in-place.
    ///
    /// ASCII letters 'a' to 'z' are mapped to 'A' to 'Z',
    /// but non-ASCII letters are unchanged.
    #[inline]
    #[cfg(feature="std")]
    pub fn make_ascii_uppercase(&mut self) {
        self.bytes[0].make_ascii_uppercase()
    }
    /// Converts the character to its ASCII lower case equivalent in-place.
    ///
    /// ASCII letters 'A' to 'Z' are mapped to 'a' to 'z',
    /// but non-ASCII letters are unchanged.
    #[inline]
    #[cfg(feature="std")]
    pub fn make_ascii_lowercase(&mut self) {
        self.bytes[0].make_ascii_lowercase();
    }

    /// Convert from UTF-8 to UTF-32
    pub fn to_char(self) -> char {
        self.into()
    }
    /// Write the internal representation to a slice,
    /// and then returns the number of bytes written.
    ///
    /// # Panics
    /// Will panic the buffer is too small;
    /// You can get the required length from `.len()`,
    /// but a buffer of length four is always large enough.
    pub fn to_slice(self,  dst: &mut[u8]) -> usize {
        if self.len() > dst.len() {
            panic!("The provided buffer is too small.");
        }
        dst[..self.len()].copy_from_slice(&self.bytes[..self.len()]);
        self.len()
    }
    /// Expose the internal array and the number of used bytes.
    pub fn to_array(self) -> ([u8;4],usize) {
        (self.bytes, self.len())
    }
    /// Return a `str` view of the array the codepoint is stored as.
    /// Ns an unambiguous version of `.as_ref()`.
    pub fn as_str(&self) -> &str {
        self.deref()
    }
}
