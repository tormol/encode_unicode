/* Copyright 2016 Torbjørn Birch Moltu
 *
 * Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
 * http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
 * http://opensource.org/licenses/MIT>, at your option. This file may not be
 * copied, modified, or distributed except according to those terms.
 */

use Utf16Iterator;
use CharExt;
use errors::{InvalidUtf16Slice,InvalidUtf16Tuple};
extern crate core;
use self::core::{hash,fmt,cmp};
use self::core::borrow::Borrow;
use self::core::ops::Deref;
#[cfg(feature="std")]
use std::ascii::AsciiExt;
#[cfg(feature="ascii")]
use self::core::char;
#[cfg(feature="ascii")]
extern crate ascii;
#[cfg(feature="ascii")]
use self::ascii::{AsciiChar,ToAsciiChar,ToAsciiCharError};


// I don't think there is any good default value for char, but char does.
#[derive(Default)]
// char doesn't do anything more advanced than u32 for Eq/Ord, so we shouldn't either.
// When it's a single unit, the second is zero, so Eq works.
// Ord however, breaks on surrogate pairs.
#[derive(PartialEq,Eq)]
#[derive(Clone,Copy)]


/// Store a `char` as UTF-16 so it can be borrowed as a slice
///
/// Size is identical to `char`.
/// Cannot represent all 2^32-1 possible values, but can do all valid ones.
pub struct Utf16Char {
    units: [u16; 2],
}


  /////////////////////
 //conversion traits//
/////////////////////

impl From<char> for Utf16Char {
    fn from(c: char) -> Self {
        let (first, second) = c.to_utf16_tuple();
        Utf16Char{ units: [first, second.unwrap_or(0)] }
    }
}
impl From<Utf16Char> for char {
    fn from(uc: Utf16Char) -> char {
        unsafe{ char::from_utf16_tuple_unchecked(uc.to_tuple()) }
    }
}
impl IntoIterator for Utf16Char {
    type Item=u16;
    type IntoIter=Utf16Iterator;
    /// Iterate over the units.
    fn into_iter(self) -> Utf16Iterator {
        Utf16Iterator::from(self)
    }
}


  /////////////////
 //getter traits//
/////////////////
impl AsRef<[u16]> for Utf16Char {
    fn as_ref(&self) -> &[u16] {
        &self.units[..self.len()]
    }
}
impl Borrow<[u16]> for Utf16Char {
    fn borrow(&self) -> &[u16] {
        self.as_ref()
    }
}
impl Deref for Utf16Char {
    type Target = [u16];
    fn deref(&self) -> &[u16] {
        self.as_ref()
    }
}


  ////////////////
 //ascii traits//
////////////////
#[cfg(feature="std")]
impl AsciiExt for Utf16Char {
    type Owned = Self;
    fn is_ascii(&self) -> bool {
        self.units[0] < 128
    }
    fn eq_ignore_ascii_case(&self,  other: &Self) -> bool {
        self.to_char().eq_ignore_ascii_case(&other.to_char())
    }
    fn to_ascii_uppercase(&self) -> Self {
        self.to_char().to_ascii_uppercase().to_utf16()
    }
    fn to_ascii_lowercase(&self) -> Self {
        self.to_char().to_ascii_lowercase().to_utf16()
    }
    // Theese methods are what prevents this from becoming stable
    fn make_ascii_uppercase(&mut self) {
        *self = self.to_ascii_uppercase()
    }
    fn make_ascii_lowercase(&mut self) {
        *self = self.to_ascii_lowercase();
    }
}

#[cfg(feature="ascii")]
/// Requires feature "ascii".
impl From<AsciiChar> for Utf16Char {
    fn from(ac: AsciiChar) -> Self {
        Utf16Char{ units: [ac.as_byte() as u16, 0] }
    }
}
#[cfg(feature="ascii")]
/// Requires feature "ascii".
impl ToAsciiChar for Utf16Char {
    fn to_ascii_char(self) -> Result<AsciiChar, ToAsciiCharError> {
        unsafe{ AsciiChar::from(char::from_u32_unchecked(self.units[0] as u32)) }
    }
    unsafe fn to_ascii_char_unchecked(self) -> AsciiChar {
        AsciiChar::from_unchecked(self.units[0] as u8)
    }
}


  /////////////////////////////////////////////////////////
 //Genaral traits that cannot be derived to emulate char//
/////////////////////////////////////////////////////////
impl hash::Hash for Utf16Char {
    fn hash<H : hash::Hasher>(&self,  state: &mut H) {
        self.to_char().hash(state);
    }
}
impl fmt::Debug for Utf16Char {
    fn fmt(&self,  fmtr: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(&self.to_char(), fmtr)
    }
}
impl cmp::PartialOrd for Utf16Char {
    fn partial_cmp(&self,  rhs: &Self) -> Option<cmp::Ordering> {
        self.to_char().partial_cmp(&rhs.to_char())
    }
}
impl cmp::Ord for Utf16Char {
    fn cmp(&self,  rhs: &Self) -> cmp::Ordering {
        self.to_char().cmp(&rhs.to_char())
    }
}


  ///////////////////////////////////////////////////////
 //pub impls that should be together for nicer rustdoc//
///////////////////////////////////////////////////////
impl Utf16Char {
    /// Validate and store the first UTF-16 codepoint in the slice.
    /// Also return how many units were needed.
    pub fn from_slice(src: &[u16]) -> Result<(Self,usize),InvalidUtf16Slice> {
        char::from_utf16_slice(src).map(|(_,len)| {
            let second = if len==2 {src[1]} else {0};
            (Utf16Char{ units: [src[0], second] }, len)
        })
    }
    /// Validate and store a UTF-16 pair as returned from `char.to_utf16_tuple()`.
    pub fn from_tuple(utf16: (u16,Option<u16>)) -> Result<Self,InvalidUtf16Tuple> {
        char::from_utf16_tuple(utf16).map(|_|
            Utf16Char{ units: [utf16.0, utf16.1.unwrap_or(0)] }
        )
    }

    /// Returns 1 or 2.
    /// There is no `.is_emty()` because it would always return false.
    pub fn len(self) -> usize {
        if self.units[1] == 0 {1} else {2}
    }

    /// Convert from UTF-16 to UTF-32
    pub fn to_char(self) -> char {
        self.into()
    }
    /// Write the internal representation to a slice,
    /// and then returns the number of `u16`s written.
    ///
    /// # Panics
    /// Will panic the buffer is too small;
    /// You can get the required length from `.len()`,
    /// but a buffer of length two is always large enough.
    pub fn to_slice(self,  dst: &mut[u16]) -> usize {
        match self.len() {
            l if l > dst.len() => panic!("The provided buffer is too small."),
            2 => {dst[1] = self.units[1];
                  dst[0] = self.units[0];},
            1 => {dst[0] = self.units[0];},
            _ => unreachable!()
        }
        self.len()
    }
    /// The second `u16` is used for surrogate pairs.
    pub fn to_tuple(self) -> (u16,Option<u16>) {
        (self.units[0],  if self.len()==2 {Some(self.units[1])} else {None})
    }
    #[cfg(not(feature="std"))]
    pub fn is_ascii(&self) -> bool {
        self.units[0] <= 127
    }
}
