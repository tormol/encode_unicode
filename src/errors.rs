/* Copyright 2016-2022 Torbjørn Birch Moltu
 *
 * Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
 * http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
 * http://opensource.org/licenses/MIT>, at your option. This file may not be
 * copied, modified, or distributed except according to those terms.
 */


//! Boilerplatey error types

extern crate core;
use core::fmt::{self,Display,Formatter};
use core::ops::RangeInclusive;
#[cfg(feature="std")]
use std::error::Error;


macro_rules! description {($err:ty, $desc:expr) => {
    #[cfg(not(feature="std"))]
    impl $err {
        #[allow(missing_docs)]
        pub fn description(&self) -> &'static str {
            ($desc)(self)
        }
    }
    #[cfg(feature="std")]
    impl Error for $err {
        fn description(&self) -> &'static str {
            ($desc)(self)
        }
    }
    impl Display for $err {
        fn fmt(&self,  fmtr: &mut Formatter) -> fmt::Result {
            #![allow(deprecated)] // calling our own function
            write!(fmtr, "{}", self.description())
        }
    }
}}


macro_rules! single_cause {($(#[$doc:meta])* $err:ident => $desc:expr) => {
    $(#[$doc])*
    #[derive(Clone,Copy, Debug, PartialEq,Eq)]
    pub struct $err;
    description!{$err, |_| $desc }
}}


single_cause!{
    /// Cannot tell whether an `u16` needs an extra unit,
    /// because it's a trailing surrogate itself.
    InvalidUtf16FirstUnit => "is a trailing surrogate"
}

single_cause!{
    /// Error returned by [`Utf8Char::from_ascii()`](struct.Utf8Char.html#method.from_ascii)
    /// for bytes that are not ASCII characters.
    NonAsciiError => "not an ASCII character"
}

single_cause!{
    /// Error returned by [`Utf16Char::from_bmp()`](struct.Utf16Char#method.from_bmp)
    /// for units that are not a standalone codepoint.
    NonBmpError => "not a codepoint in the basic multilingual plane"
}

single_cause!{
    /// Cannot create an `Utf8Char` or `Utf16Char` from the first codepoint of a str,
    /// because there are none.
    EmptyStrError => "is empty"
}



macro_rules! simple {($(#[$tydoc:meta])* $err:ident {
                          $( $(#[$vardoc:meta])* $variant:ident => $string:expr, )+
                      } ) => {
    $(#[$tydoc])*
    #[derive(Clone,Copy, Debug, PartialEq,Eq)]
    pub enum $err {
        $( $(#[$vardoc])* $variant, )*
    }
    description!{$err, |e: &$err| match *e {$($err::$variant => $string),*} }
}}


simple!{
    /// Reasons why an `u32` is not a valid unicode codepoint. The enum values are not stable
    CodepointError {
        /// It's reserved for UTF-16 surrogate pairs.
        Utf16Reserved => "is reserved for UTF-16 surrogate pairs",
        /// It's higher than the highest codepoint (which is 0x10ffff).
        TooHigh => "is higher than the highest codepoint",
    }}
use CodepointError::*;
impl CodepointError {
    /// Get the range of values for which this error would be given.
    pub fn error_range(self) -> RangeInclusive<u32> {match self {
        Utf16Reserved => 0xd8_00..=0xdf_ff,
        TooHigh => 0x00_10_ff_ff..=0xff_ff_ff_ff,
    }}
}


simple!{/// Reasons why a `[u16; 2]` doesn't form a valid UTF-16 codepoint.
    InvalidUtf16Array {
        /// The first element is a trailing / low surrogate, which is never valid.
        FirstIsTrailingSurrogate => "the first element is a trailing surrogate",
        /// The second element is needed, but is not a trailing surrogate.
        SecondIsNotTrailingSurrogate => "the second element is needed but is not a trailing surrogate",
    }}

simple!{/// Reasons why one or two `u16`s are not valid UTF-16, in sinking precedence.
    InvalidUtf16Tuple {
        /// The first unit is a trailing / low surrogate, which is never valid.
        FirstIsTrailingSurrogate => "the first unit is a trailing surrogate",
        /// The provided second elemented is not necessary.
        SuperfluousSecond => "the second unit is superfluous",
        /// The first and only unit requires a second unit.
        MissingSecond => "the first unit requires a second unit",
        /// The second unit is needed and was provided, but is not a trailing surrogate.
        SecondIsNotTrailingSurrogate => "the required second unit is not a trailing surrogate",
    }}


simple!{/// Reasons why a slice of `u16`s doesn't start with valid UTF-16.
    InvalidUtf16Slice {
        /// The slice is empty.
        EmptySlice => "the slice is empty",
        /// The first unit is a trailing surrogate.
        FirstIsTrailingSurrogate => "the first unit is a trailing surrogate",
        /// The first and only unit requires a second unit.
        MissingSecond => "the first and only unit requires a second one",
        /// The first unit requires a second one, but it's not a trailing surrogate.
        SecondIsNotTrailingSurrogate => "the required second unit is not a trailing surrogate",
    }}

simple!{/// Types of invalid sequences encountered by `Utf16CharDecoder`.
    Utf16PairError {
        /// A trailing surrogate was not preceeded by a leading surrogate.
        UnexpectedTrailingSurrogate => "a trailing surrogate was not preceeded by a leading surrogate",
        /// A leading surrogate was followed by an unit that was not a trailing surrogate.
        UnmatchedLeadingSurrogate => "a leading surrogate was followed by an unit that was not a trailing surrogate",
        /// A trailing surrogate was expected when the end was reached.
        Incomplete => "a trailing surrogate was expected when the end was reached",
    }}


simple!{/// Reasons why `Utf8Char::from_str()` or `Utf16Char::from_str()` failed.
    FromStrError {
        /// `Utf8Char` and `Utf16Char` cannot store more than a single codepoint.
        MultipleCodepoints => "Contains more than one codepoint",
        /// `Utf8Char` and `Utf16Char` cannot be empty.
        Empty => "Is empty",
    }
}



/// Error returned when an invalid UTF-8 sequence is encountererd.
///
/// See [`Utf8ErrorKind`](enum.Utf8ErrorKind.html) for the types of errors
/// that this type can be returned for.
#[derive(Clone,Copy, Debug, PartialEq,Eq)]
pub struct Utf8Error {
    pub(crate) kind: Utf8ErrorKind,
}
impl Utf8Error {
    /// Get the type of error.
    pub fn kind(&self) -> Utf8ErrorKind {
        self.kind
    }

    #[cfg(not(feature="std"))]
    #[allow(missing_docs)]
    pub fn description(&self) -> &'static str {
        utf8_error_description(self.kind)
    }
}
#[cfg(feature="std")]
impl Error for Utf8Error {
    fn description(&self) -> &'static str {
        utf8_error_description(self.kind)
    }
}
impl Display for Utf8Error {
    fn fmt(&self,  fmtr: &mut Formatter) -> fmt::Result {
        fmtr.write_str(utf8_error_description(self.kind))
    }
}

/// The types of errors that can occur when decoding a UTF-8 codepoint.
///
/// The variants are more technical than what an end user is likely interested
/// in, but might be useful for deciding how to handle the error.
///
/// They can be grouped into three categories:
/// * Will happen regularly if decoding chunked or buffered text: `TooFewBytes`.
/// * Input might be binary, a different encoding or corrupted, `UnexpectedContinuationByte`
///   and `InterruptedSequence`.  
///   (Broken UTF-8 sequence).
/// * Less likely to happen accidentaly and might be malicious:
///   `OverlongEncoding`, `Utf16ReservedCodepoint` and `TooHighCodepoint`.
///   Note that theese can still be caused by certain valid latin-1 strings
///   such as `"Á©"` (`b"\xC1\xA9"`).
#[derive(Clone,Copy, Debug, PartialEq,Eq)]
pub enum Utf8ErrorKind {
    /// There are too few bytes to decode the codepoint.
    ///
    /// This can happen when a slice is empty or too short, or an iterator
    /// returned `None` while in the middle of a codepoint.  
    /// This error is never produced by functions accepting fixed-size
    /// `[u8; 4]` arrays.
    ///
    /// If decoding text coming chunked (such as in buffers passed to `Read`),
    /// the remaing bytes should be carried over into the next chunk or buffer.
    /// (including the byte this error was produced for.)
    TooFewBytes,
    /// A byte which is never used by well-formed UTF-8 was encountered.
    ///
    /// This means that the input is using a different encoding,
    /// is corrupted or binary.
    ///
    /// This error is returned when a byte in the following ranges
    /// is encountered anywhere in an UTF-8 sequence:
    ///
    /// * `192` and `193` (`0b1100_000x`): Indicates an overlong encoding
    ///   of a single-byte, ASCII, character, and should therefore never occur.
    /// * `248..` (`0b1111_1xxx`): Sequences cannot be longer than 4 bytes.
    /// * `245..=247` (`0b1111_0101 | 0b1111_0110`): Indicates a too high
    ///   codepoint. (above `\u10ffff`)
    NonUtf8Byte,
    /// The first byte is not a valid start of a codepoint.
    ///
    /// This might happen as a result of slicing into the middle of a codepoint,
    /// the input not being UTF-8 encoded or being corrupted.
    /// Errors of this type coming right after another error should probably
    /// be ignored, unless returned more than three times in a row.
    ///
    /// This error is returned when the first byte has a value in the range
    /// `128..=191` (`0b1000_0000..=0b1011_1111`).
    UnexpectedContinuationByte,
    /// The byte at index 1..=3 should be a continuation byte,
    /// but doesn't fit the pattern `0b10xx_xxxx`.
    ///
    /// When the input slice or iterator has too few bytes,
    /// [`TooFewBytes`](#Incomplete) is returned instead.
    InterruptedSequence,
    /// The encoding of the codepoint has so many leading zeroes that it
    /// could be a byte shorter.
    ///
    /// [Successfully decoding this can present a security issue](https://tools.ietf.org/html/rfc3629#section-10):
    /// Doing so could allow an attacker to circumvent input validation that
    /// only checks for ASCII characters, and input characters or strings that
    /// would otherwise be rejected, such as `/../`.
    ///
    /// This error is only returned for 3 and 4-byte encodings;
    /// `NonUtf8Byte` is returned for bytes that start longer or shorter
    /// overlong encodings.
    OverlongEncoding,
    /// The codepoint is reserved for UTF-16 surrogate pairs.
    ///
    /// (`Utf8Char` cannot be used to work with the
    /// [WTF-8](https://simonsapin.github.io/wtf-8) encoding for UCS-2 strings.)
    ///
    /// This error is returned for codepoints in the range `\ud800`..=`\udfff`.
    /// (which are three bytes long as UTF-8)
    Utf16ReservedCodepoint,
    /// The codepoint is higher than `\u10ffff`, which is the highest codepoint
    /// unicode permits.
    TooHighCodepoint,
}
fn utf8_error_description(kind: Utf8ErrorKind) -> &'static str {
    match kind {
        Utf8ErrorKind::TooFewBytes => "Too few bytes",
        Utf8ErrorKind::NonUtf8Byte => "Not UTF-8",
        Utf8ErrorKind::UnexpectedContinuationByte => "Not UTF-8",
        Utf8ErrorKind::InterruptedSequence => "Not UTF-8",
        Utf8ErrorKind::OverlongEncoding => "Malformed input",
        Utf8ErrorKind::Utf16ReservedCodepoint => "Malformed input",
        Utf8ErrorKind::TooHighCodepoint => "Invalid character",
    }
}
impl PartialEq<Utf8ErrorKind> for Utf8Error {
    fn eq(&self,  kind: &Utf8ErrorKind) -> bool {
        self.kind == *kind
    }
}
impl PartialEq<Utf8Error> for Utf8ErrorKind {
    fn eq(&self,  error: &Utf8Error) -> bool {
        *self == error.kind
    }
}
