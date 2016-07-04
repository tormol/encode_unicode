/* Copyright 2016 Torbjørn Birch Moltu
 *
 * Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
 * http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
 * http://opensource.org/licenses/MIT>, at your option. This file may not be
 * copied, modified, or distributed except according to those terms.
 */


//! Boilerplatey error enums


extern crate std;
use std::fmt::{self,Display,Formatter};
use std::error::Error;



/// Reasons why `Utf8Char::from_str()` failed.
#[derive(Clone,Copy, Debug, PartialEq,Eq)]
pub enum FromStrError {
    /// `Utf8Char` cannot store more than a single codepoint.
    SeveralCodePoints,
    /// `Utf8Char` cannot be empty.
    Empty,
}
use self::FromStrError::*;
impl Error for FromStrError {
    fn description(&self) -> &'static str {match *self {
        SeveralCodePoints => "has more than one codepoint",
        Empty => "is empty",
    }}
}
impl Display for FromStrError {
    fn fmt(&self,  fmtr: &mut Formatter) -> fmt::Result {
        write!(fmtr, "{}", self.description())
    }
}



/// Reasons why an `u32` is not a valid UTF codepoint.
#[derive(Clone,Copy, Debug, PartialEq,Eq)]
pub enum InvalidCodePoint {
    /// It's reserved for UTF-16 surrogate pairs.
    Utf16Reserved,
    /// It's higher than the highest codepoint of 0x10ffff.
    TooHigh,
}
use self::InvalidCodePoint::*;
impl InvalidCodePoint {
    /// Get the range of values for which this error would be given.
    pub fn error_range(self) -> (u32,u32) {match self {
        Utf16Reserved => (0xd8_00, 0xdf_ff),
        TooHigh => (0x00_10_ff_ff, 0xff_ff_ff_ff),
    }}
}
impl Error for InvalidCodePoint {
    fn description(&self) -> &'static str {match *self {
        Utf16Reserved => "is reserved for UTF-16 surrogate pairs",
        TooHigh => "is higher than the highest codepoint of 0x10ffff",
    }}
}
impl Display for InvalidCodePoint {
    fn fmt(&self,  fmtr: &mut Formatter) -> fmt::Result {
        write!(fmtr, "{}", self.description())
    }
}



/// Reasons why a byte is not the start of a UTF-8 codepoint.
#[derive(Clone,Copy, Debug, PartialEq,Eq)]
pub enum InvalidUtf8FirstByte {
    /// Sequences cannot be longer than 4 bytes. Is given for values >= 240.
    TooLongSeqence,
    /// This byte belongs to a previous seqence. Is given for values between 128 and 192 (exclusive).
    ContinuationByte,
}
use self::InvalidUtf8FirstByte::*;
impl Error for InvalidUtf8FirstByte {
    fn description(&self) -> &'static str {match *self {
        TooLongSeqence => "is greater than 239 (UTF-8 seqences cannot be longer than four bytes)",
        ContinuationByte => "is a continuation of a previous sequence",
    }}
}
impl Display for InvalidUtf8FirstByte {
    fn fmt(&self,  fmtr: &mut Formatter) -> fmt::Result {
        write!(fmtr, "{}", self.description())
    }
}


/// Reasons why a byte sequence is not valid UTF-8, excluding invalid codepoint.
/// In sinking precedence.
#[derive(Clone,Copy, Debug, PartialEq,Eq)]
pub enum InvalidUtf8 {
    /// Something is wrong with the first byte.
    FirstByte(InvalidUtf8FirstByte),
    /// Thee byte at index 1...3 should be a continuation byte,
    /// but dosesn't fit the pattern 0b10xx_xxxx.
    NotAContinuationByte(usize),
    /// There are too many leading zeros: it could be a byte shorter.
    OverLong,
}
use self::InvalidUtf8::*;
impl From<InvalidUtf8FirstByte> for InvalidUtf8 {
    fn from(error: InvalidUtf8FirstByte) -> InvalidUtf8 {
        FirstByte(error)
    }
}
impl Error for InvalidUtf8 {
    fn description(&self) -> &'static str {match *self {
        FirstByte(TooLongSeqence) => "the first byte is greater than 239 (UTF-8 seqences cannot be longer than four bytes)",
        FirstByte(ContinuationByte) => "the first byte is a continuation of a previous sequence",
        OverLong => "the seqence contains too many zeros and could be shorter",
        NotAContinuationByte(_) => "the sequence is too short",
    }}
    /// When `Some` is returned, the `Error` is a `InvalidUtf8FirstByte`.
    fn cause(&self) -> Option<&Error> {match *self {
        FirstByte(ref cause) => Some(cause),
        _ => None,
    }}
}
impl Display for InvalidUtf8 {
    fn fmt(&self,  fmtr: &mut Formatter) -> fmt::Result {
        write!(fmtr, "{}", self.description())
    }
}


/// Reasons why a byte array is not valid UTF-8, in sinking precedence.
#[derive(Clone,Copy, Debug, PartialEq,Eq)]
pub enum InvalidUtf8Array {
    /// Not a valid UTF-8 sequence.
    Utf8(InvalidUtf8),
    /// Not a valid unicode codepoint.
    CodePoint(InvalidCodePoint),
}
impl From<InvalidUtf8> for InvalidUtf8Array {
    fn from(error: InvalidUtf8) -> InvalidUtf8Array {
        InvalidUtf8Array::Utf8(error)
    }
}
impl From<InvalidCodePoint> for InvalidUtf8Array {
    fn from(error: InvalidCodePoint) -> InvalidUtf8Array {
        InvalidUtf8Array::CodePoint(error)
    }
}
impl Error for InvalidUtf8Array {
    fn description(&self) -> &'static str {match *self {
        InvalidUtf8Array::Utf8(_) => "the seqence is invalid UTF-8",
        InvalidUtf8Array::CodePoint(_) => "the encoded codepoint is invalid",
    }}
    /// Always returns `Some`.
    fn cause(&self) -> Option<&Error> {match *self {
        InvalidUtf8Array::Utf8(ref u) => Some(u),
        InvalidUtf8Array::CodePoint(ref c) => Some(c),
    }}
}
impl Display for InvalidUtf8Array {
    fn fmt(&self,  fmtr: &mut Formatter) -> fmt::Result {
        write!(fmtr, "{}: {}", self.description(), self.cause().unwrap().description())
    }
}


/// Reasons why a byte slice is not valid UTF-8, in sinking precedence.
#[derive(Clone,Copy, Debug, PartialEq,Eq)]
pub enum InvalidUtf8Slice {
    /// Something is certainly wrong with the first byte.
    Utf8(InvalidUtf8),
    /// The encoded codepoint is invalid:
    CodePoint(InvalidCodePoint),
    /// The slice is too short; n bytes was required.
    TooShort(usize),
}
impl From<InvalidUtf8> for InvalidUtf8Slice {
    fn from(error: InvalidUtf8) -> InvalidUtf8Slice {
        InvalidUtf8Slice::Utf8(error)
    }
}
impl From<InvalidCodePoint> for InvalidUtf8Slice {
    fn from(error: InvalidCodePoint) -> InvalidUtf8Slice {
        InvalidUtf8Slice::CodePoint(error)
    }
}
impl Error for InvalidUtf8Slice {
    fn description(&self) -> &'static str {match *self {
        InvalidUtf8Slice::Utf8(_) => "the seqence is invalid UTF-8",
        InvalidUtf8Slice::CodePoint(_) => "the encoded codepoint is invalid",
        InvalidUtf8Slice::TooShort(0) => "the slice is empty",
        InvalidUtf8Slice::TooShort(_) => "the slice is shorter than the seqence",
    }}
    fn cause(&self) -> Option<&Error> {match *self {
        InvalidUtf8Slice::Utf8(ref u) => Some(u),
        InvalidUtf8Slice::CodePoint(ref c) => Some(c),
        InvalidUtf8Slice::TooShort(_) => None,
    }}
}
impl Display for InvalidUtf8Slice {
    fn fmt(&self,  fmtr: &mut Formatter) -> fmt::Result {
        match self.cause() {
            Some(d) => write!(fmtr, "{}: {}", self.description(), d),
            None    => write!(fmtr, "{}", self.description()),
        }
    }
}



/// Reasons why one or two `u16`s are not valid UTF-16, in sinking precedence.
#[derive(Clone,Copy, Debug, PartialEq,Eq)]
pub enum InvalidUtf16Tuple {
    /// The first unit is a trailing/low surrogate, which is never valid.
    ///
    /// Note that the value of a low surrogate is actually higher than a high surrogate.
    FirstIsTrailingSurrogate,
    /// You provided a second unit, but the first one stands on its own.
    SuperfluousSecond,
    /// The first and only unit requires a second unit.
    MissingSecond,
    /// The first unit requires a second unit, but it's not a trailing/low surrogate.
    ///
    /// Note that the value of a low surrogate is actually higher than a high surrogate.
    InvalidSecond,
}
impl Error for InvalidUtf16Tuple {
    fn description(&self) -> &'static str {match *self {
        InvalidUtf16Tuple::FirstIsTrailingSurrogate => "the first unit is a trailing / low surrogate, which is never valid",
        InvalidUtf16Tuple::SuperfluousSecond => "the second unit is superfluous",
        InvalidUtf16Tuple::MissingSecond => "the first unit requires a second unit",
        InvalidUtf16Tuple::InvalidSecond => "the required second unit is not a trailing / low surrogate",
    }}
}
impl Display for InvalidUtf16Tuple {
    fn fmt(&self,  fmtr: &mut Formatter) -> fmt::Result {
        write!(fmtr, "{}", self.description())
    }
}


/// Reasons why a slice of `u16`s doesn't start with valid UTF-16.
#[derive(Clone,Copy, Debug, PartialEq,Eq)]
pub enum InvalidUtf16Slice {
    /// The slice is empty.
    EmptySlice,
    /// The first unit is a low surrogate.
    FirstLowSurrogate,
    /// The first and only unit requires a second unit.
    MissingSecond,
    /// The first unit requires a second one, but it's not a low surrogate.
    SecondNotLowSurrogate,
}
impl Error for InvalidUtf16Slice {
    fn description(&self) -> &'static str {match *self {
        InvalidUtf16Slice::EmptySlice => "the slice is empty",
        InvalidUtf16Slice::FirstLowSurrogate => "the first unit is a low surrogate",
        InvalidUtf16Slice::MissingSecond => "the first and only unit requires a second one",
        InvalidUtf16Slice::SecondNotLowSurrogate => "the required second unit is not a low surrogate",
    }}
}
impl Display for InvalidUtf16Slice {
    fn fmt(&self,  fmtr: &mut Formatter) -> fmt::Result {
        write!(fmtr, "{}", self.description())
    }
}
