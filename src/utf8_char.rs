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
extern crate core;
use self::core::{hash, fmt, str};
use self::core::borrow::Borrow;
use self::core::ops::Deref;
use self::core::mem::transmute;
#[cfg(not(feature="no_std"))]
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


/// Store a `char` as UTF-8 so it can be borrowed as a `str`
///
/// Has the same size as `char`, and is a `[u8;4]`
/// with the invariant that the first nth bytes are valid UTF-8, and the remaining are zero.
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
            Err(FromStrError::SeveralCodePoints)
        } else {
            let mut bytes = [0; 4];
            bytes[..s.len()].copy_from_slice(s.as_bytes());
            Ok(Utf8Char{bytes: bytes})
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
#[cfg(not(feature="no_std"))]
impl AsciiExt for Utf8Char {
    type Owned = Utf8Char;
    fn is_ascii(&self) -> bool {
        self.bytes[0].is_ascii()
    }
    fn eq_ignore_ascii_case(&self,  other: &Self) -> bool {
        self.to_char().eq_ignore_ascii_case(&other.to_char())
    }
    fn to_ascii_uppercase(&self) -> Self::Owned {
        let ascii = self.bytes[0].to_ascii_uppercase();
        if ascii == self.bytes[0] {*self}
        else {Utf8Char{ bytes: [ascii,0,0,0] }}
    }
    fn to_ascii_lowercase(&self) -> Self::Owned {
        let ascii = self.bytes[0].to_ascii_lowercase();
        if ascii == self.bytes[0] {*self}// unchanged
        else {Utf8Char{ bytes: [ascii,0,0,0] }}// is ascii
    }
    fn make_ascii_uppercase(&mut self) {
        *self = self.to_ascii_uppercase()
    }
    fn make_ascii_lowercase(&mut self) {
        *self = self.to_ascii_lowercase();
    }
}

#[cfg(feature="ascii")]
/// Requires feature "ascii".
impl From<AsciiChar> for Utf8Char {
    fn from(ac: AsciiChar) -> Self {
        Utf8Char{ bytes: [ac.as_byte(),0,0,0] }
    }
}
#[cfg(feature="ascii")]
/// Requires feature "asciiChar".
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
        // Converts to char to check codepoint, but we don't need the char.
        let len = try!(char::from_utf8_slice(src)).1;
        let mut bytes = [0; 4];
        bytes[..len].copy_from_slice(&src[..len]);
        Ok((Utf8Char{bytes: bytes}, len))
    }
    /// Validate the array and store it.
    pub fn from_array(utf8: [u8;4]) -> Result<Self,InvalidUtf8Array> {
        try!(char::from_utf8_array(utf8));
        let extra = utf8[0].extra_utf8_bytes_unchecked() as u32;
        let mask = u32::from_le(0xff_ff_ff_ff >> 8*(3-extra));
        let unused_zeroed = mask  &  unsafe{ transmute::<_,u32>(utf8) };
        Ok(unsafe{ transmute(unused_zeroed) })
    }

    /// Result is 1...4 and identical to `.as_ref().len()` or `.as_char().len_utf8()`.
    /// There is no .is_emty() because it would always return false.
    pub fn len(self) -> usize {
        self.bytes[0].extra_utf8_bytes_unchecked() + 1
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
}
