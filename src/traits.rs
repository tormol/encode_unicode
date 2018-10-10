/* Copyright 2016 The encode_unicode Developers
 *
 * Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
 * http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
 * http://opensource.org/licenses/MIT>, at your option. This file may not be
 * copied, modified, or distributed except according to those terms.
 */

#![allow(unused_unsafe)]// explicit unsafe{} blocks in unsafe functions are a good thing.

use utf8_char::Utf8Char;
use utf16_char::Utf16Char;
use utf8_iterators::*;
use utf16_iterators::*;
use error::*;
extern crate core;
use self::core::{char, u32, mem};
use self::core::ops::Not;
use self::core::borrow::Borrow;
#[cfg(feature="ascii")]
extern crate ascii;
#[cfg(feature="ascii")]
use self::ascii::AsciiStr;

// TODO better docs and tests

/// Methods for working with `u8`s as UTF-8 bytes.
pub trait U8UtfExt {
    /// How many more bytes will you need to complete this codepoint?
    ///
    /// # Errors
    ///
    /// An error is returned if the byte is not a valid start of an UTF-8
    /// codepoint:
    ///
    /// * `128..192`: ContinuationByte
    /// * `248..`: TooLongSequence
    ///
    /// Values in 244..248 represent a too high codepoint, but do not cause an
    /// error.
    fn extra_utf8_bytes(self) -> Result<usize,InvalidUtf8FirstByte>;

    /// How many more bytes will you need to complete this codepoint?
    ///
    /// This function assumes that the byte is a valid UTF-8 start, and might
    /// return any value otherwise. (but the function is pure and safe to call
    /// with any value).
    fn extra_utf8_bytes_unchecked(self) -> usize;
}

impl U8UtfExt for u8 {
    #[inline]
    fn extra_utf8_bytes(self) -> Result<usize,InvalidUtf8FirstByte> {
        use error::InvalidUtf8FirstByte::{ContinuationByte,TooLongSeqence};
        // the bit twiddling is explained in extra_utf8_bytes_unchecked()
        if self < 128 {
            return Ok(0);
        }
        match ((self as u32)<<25).not().leading_zeros() {
            n @ 1...3 => Ok(n as usize),
            0 => Err(ContinuationByte),
            _ => Err(TooLongSeqence),
        }
    }
    #[inline]
    fn extra_utf8_bytes_unchecked(self) -> usize {
        // For fun I've optimized this function (for x86 instruction count):
        // The most straightforward implementation, that lets the compiler do
        // the optimizing:
        //match self {
        //    0b0000_0000...0b0111_1111 => 0,
        //    0b1100_0010...0b1101_1111 => 1,
        //    0b1110_0000...0b1110_1111 => 2,
        //    0b1111_0000...0b1111_0100 => 3,
        //                _             => whatever()
        //}
        // Using `unsafe{self::core::hint::unreachable_unchecked()}` for the
        // "don't care" case is a terrible idea: while having the function
        // non-deterministically return whatever happens to be in a register
        // MIGHT be acceptable, it permits the function to not `ret`urn at all,
        // but let execution fall through to whatever comes after it in the
        // binary! (in other words completely UB).
        // Currently unreachable_unchecked() might trap too,
        // which is certainly not what we want.
        // I also think `unsafe{mem::unitialized()}` is much more likely to
        // explicitly produce whatever happens to be in a register than tell
        // the compiler it can ignore this branch but needs to produce a value.
        //
        // From the bit patterns we see that for non-ASCII values the result is
        // (number of leading set bits) - 1
        // The standard library doesn't have a method for counting leading ones,
        // but it has leading_zeros(), which can be used after inverting.
        // This function can therefore be reduced to the one-liner
        //`self.not().leading_zeros().saturating_sub(1) as usize`, which would
        // be branchless for architectures with instructions for
        // leading_zeros() and saturating_sub().

        // Shortest version as long as ASCII-ness can be predicted: (especially
        // if the BSR instruction which leading_zeros() uses is microcoded or
        // doesn't exist)
        // u8.leading_zeros() would cast to a bigger type internally, so that's
        // free. compensating by shifting left by 24 before inverting lets the
        // compiler know that the value passed to leading_zeros() is not zero,
        // for which BSR's output is undefined and leading_zeros() normally has
        // special case with a branch.
        // Shifting one bit too many left acts as a saturating_sub(1).
        if self<128 {0} else {((self as u32)<<25).not().leading_zeros() as usize}

        // Branchless but longer version: (9 instructions)
        // It's tempting to try (self|0x80).not().leading_zeros().wrapping_sub(1),
        // but that produces high lengths for ASCII values 0b01xx_xxxx.
        // If we could somehow (branchlessy) clear that bit for ASCII values...
        // We can by masking with the value shifted right with sign extension!
        // (any nonzero number of bits in range works)
        //let extended = self as i8 as i32;
        //let ascii_cleared = (extended<<25) & (extended>>25);
        //ascii_cleared.not().leading_zeros() as usize

        // cmov version: (7 instructions)
        //(((self as u32)<<24).not().leading_zeros() as usize).saturating_sub(1)
    }
}


/// Methods for working with `u16`s as UTF-16 units.
pub trait U16UtfExt {
    /// Will you need an extra unit to complete this codepoint?
    ///
    /// Returns `Err` for trailing surrogates, `Ok(true)` for leading surrogates,
    /// and `Ok(false)` for others.
    fn utf16_needs_extra_unit(self) -> Result<bool,InvalidUtf16FirstUnit>;

    /// Does this `u16` need another `u16` to complete a codepoint?
    /// Returns `(self & 0xfc00) == 0xd800`
    ///
    /// Is basically an unchecked variant of `utf16_needs_extra_unit()`.
    fn is_utf16_leading_surrogate(self) -> bool;
}
impl U16UtfExt for u16 {
    #[inline]
    fn utf16_needs_extra_unit(self) -> Result<bool,InvalidUtf16FirstUnit> {
        match self {
            // https://en.wikipedia.org/wiki/UTF-16#U.2B10000_to_U.2B10FFFF
            0x00_00...0xd7_ff | 0xe0_00...0xff_ff => Ok(false),
            0xd8_00...0xdb_ff => Ok(true),
                    _         => Err(InvalidUtf16FirstUnit)
        }
    }
    #[inline]
    fn is_utf16_leading_surrogate(self) -> bool {
        (self & 0xfc00) == 0xd800// Clear the ten content bytes of a surrogate,
                                 // and see if it's a leading surrogate.
    }
}




/// Extension trait for `char` that adds methods for converting to and from UTF-8 or UTF-16.
pub trait CharExt: Sized {
    /// Get the UTF-8 representation of this codepoint.
    ///
    /// `Utf8Char` is to `[u8;4]` what `char` is to `u32`:
    /// a restricted type that cannot be mutated internally.
    fn to_utf8(self) -> Utf8Char;

    /// Get the UTF-16 representation of this codepoint.
    ///
    /// `Utf16Char` is to `[u16;2]` what `char` is to `u32`:
    /// a restricted type that cannot be mutated internally.
    fn to_utf16(self) -> Utf16Char;

    /// Iterate over or [read](https://doc.rust-lang.org/std/io/trait.Read.html)
    /// the one to four bytes in the UTF-8 representation of this codepoint.
    ///
    /// An identical alternative to the unstable `char.encode_utf8()`.
    /// That method somehow still exist on stable, so I have to use a different name.
    fn iter_utf8_bytes(self) -> Utf8Iterator;

    /// Iterate over the one or two units in the UTF-16 representation of this codepoint.
    ///
    /// An identical alternative to the unstable `char.encode_utf16()`.
    /// That method somehow still exist on stable, so I have to use a different name.
    fn iter_utf16_units(self) -> Utf16Iterator;


    /// Convert this char to an UTF-8 array and lenght,
    ///
    /// The returned array is left-aligned with unused bytes set to zero,
    /// and the usize is how many bytes are used.
    fn to_utf8_array(self) -> ([u8; 4], usize);

    /// Convert this char to UTF-16.
    /// The second `u16` is `Some` if a surrogate pair is required.
    fn to_utf16_tuple(self) -> (u16, Option<u16>);



    /// Create a `char` from the start of a UTF-8 slice,
    /// and also return how many bytes were used.
    fn from_utf8_slice_start(src: &[u8]) -> Result<(Self,usize),InvalidUtf8Slice>;

    /// Create a `char` from the start of a UTF-16 slice,
    /// and also return how many units were used.
    ///
    /// If you want to continue after an error, continue with the next `u16`.
    fn from_utf16_slice_start(src: &[u16]) -> Result<(Self,usize), InvalidUtf16Slice>;


    /// Convert an UTF-8 sequence as returned from `.to_utf8_array()` into a `char`
    fn from_utf8_array(utf8: [u8; 4]) -> Result<Self,InvalidUtf8Array>;

    /// Convert a UTF-16 pair as returned from `.to_utf16_tuple()` into a `char`.
    fn from_utf16_tuple(utf16: (u16, Option<u16>)) -> Result<Self, InvalidUtf16Tuple>;


    /// Convert an UTF-8 sequence into a char.
    /// The length of the slice is the length of the sequence, should be 1,2,3 or 4.
    ///
    /// # Panics
    /// If the slice is empty
    unsafe fn from_utf8_exact_slice_unchecked(src: &[u8]) -> Self;

    /// Convert a UTF-16 tuple as returned from `.to_utf16_tuple()` into a `char`.
    unsafe fn from_utf16_tuple_unchecked(utf16: (u16, Option<u16>)) -> Self;


    /// Perform some extra validations compared to `char::from_u32_unchecked()`
    ///
    /// # Errors
    /// This function will return an error if
    /// * the value is greater than 0x10ffff
    /// * the value is between 0xd800 and 0xdfff (inclusive)
    fn from_u32_detailed(c: u32) -> Result<Self,InvalidCodepoint>;
}



impl CharExt for char {
      /////////
     //UTF-8//
    /////////

    fn to_utf8(self) -> Utf8Char {
        self.into()
    }
    fn iter_utf8_bytes(self) -> Utf8Iterator {
        self.to_utf8().into_iter()
    }

    fn to_utf8_array(self) -> ([u8; 4], usize) {
        let len = self.len_utf8();
        let mut c = self as u32;
        if len == 1 {// ASCII, the common case
            ([c as u8, 0, 0, 0],  1)
        } else {
            let mut parts = 0;// convert to 6-bit bytes
                        parts |= c & 0x3f;  c>>=6;
            parts<<=8;  parts |= c & 0x3f;  c>>=6;
            parts<<=8;  parts |= c & 0x3f;  c>>=6;
            parts<<=8;  parts |= c & 0x3f;
            parts |= 0x80_80_80_80;// set the most significant bit
            parts >>= 8*(4-len);// right-align bytes
            // Now, unused bytes are zero, (which matters for Utf8Char.eq())
            // and the rest are 0b10xx_xxxx

            // set header on first byte
            parts |= (0xff_00u32 >> len)  &  0xff;// store length
            parts &= Not::not(1u32 << 7-len);// clear the next bit after it

            let bytes: [u8; 4] = unsafe{ mem::transmute(u32::from_le(parts)) };
            (bytes, len)
        }
    }


    fn from_utf8_slice_start(src: &[u8]) -> Result<(Self,usize),InvalidUtf8Slice> {
        use errors::InvalidUtf8::*;
        use errors::InvalidUtf8Slice::*;
        let first = match src.first() {
            Some(first) => *first,
            None => return Err(TooShort(1)),
        };
        let bytes = match first.extra_utf8_bytes() {
            Err(e)    => return Err(Utf8(FirstByte(e))),
            Ok(0)     => return Ok((first as char, 1)),
            Ok(extra) if extra >= src.len()
                      => return Err(TooShort(extra+1)),
            Ok(extra) => &src[..extra+1],
        };
        if let Some(i) = bytes.iter().skip(1).position(|&b| (b >> 6) != 0b10 ) {
            Err(Utf8(NotAContinuationByte(i+1)))
        } else if overlong(bytes[0], bytes[1]) {
            Err(Utf8(OverLong))
        } else {
            let c = unsafe{ char::from_utf8_exact_slice_unchecked(bytes) };
            match char::from_u32_detailed(c as u32) {
                Ok(c) => Ok((c, bytes.len())),
                Err(e) => Err(Codepoint(e)),
            }
        }
    }

    fn from_utf8_array(utf8: [u8; 4]) -> Result<Self,InvalidUtf8Array> {
        use errors::InvalidUtf8::*;
        use errors::InvalidUtf8Array::*;
        let src = match utf8[0].extra_utf8_bytes() {
            Err(error) => return Err(Utf8(FirstByte(error))),
            Ok(0)      => return Ok(utf8[0] as char),
            Ok(extra)  => &utf8[..extra+1],
        };
        if let Some(i) = src[1..].iter().position(|&b| (b >> 6) != 0b10 ) {
            Err(Utf8(NotAContinuationByte(i+1)))
        } else if overlong(utf8[0], utf8[1]) {
            Err(Utf8(OverLong))
        } else {
            let c = unsafe{ char::from_utf8_exact_slice_unchecked(src) };
            char::from_u32_detailed(c as u32)
                 .map_err(|e| Codepoint(e) )
        }
    }

    unsafe fn from_utf8_exact_slice_unchecked(src: &[u8]) -> Self {
        if src.len() == 1 {
            src[0] as char
        } else {
            let mut c = src[0] as u32 & (0xff >> 2+src.len()-1);
            for b in &src[1..] {
                c = (c << 6)  |  (b & 0b0011_1111) as u32;
            }
            unsafe{ char::from_u32_unchecked(c) }
        }
    }



      //////////
     //UTF-16//
    //////////

    fn to_utf16(self) -> Utf16Char {
        Utf16Char::from(self)
    }
    fn iter_utf16_units(self) -> Utf16Iterator {
        self.to_utf16().into_iter()
    }

    fn to_utf16_tuple(self) -> (u16, Option<u16>) {
        let c = self as u32;
        if c <= 0x_ff_ff {// single (or reserved, which we ignore)
            (c as u16, None)
        } else {// double (or too high, which we ignore)
            let c = c - 0x_01_00_00;
            let high = 0x_d8_00 + (c >> 10);
            let low = 0x_dc_00 + (c & 0x_03_ff);
            (high as u16,  Some(low as u16))
        }
    }


    fn from_utf16_slice_start(src: &[u16]) -> Result<(Self,usize), InvalidUtf16Slice> {
        use errors::InvalidUtf16Slice::*;
        unsafe {match (src.get(0), src.get(1)) {
            (Some(&u @ 0x00_00...0xd7_ff), _) |
            (Some(&u @ 0xe0_00...0xff_ff), _)
                => Ok((char::from_u32_unchecked(u as u32), 1)),
            (Some(&0xdc_00...0xdf_ff), _) => Err(FirstLowSurrogate),
            (None, _) => Err(EmptySlice),
            (Some(&f @ 0xd8_00...0xdb_ff), Some(&s @ 0xdc_00...0xdf_ff))
                => Ok((char::from_utf16_tuple_unchecked((f, Some(s))), 2)),
            (Some(&0xd8_00...0xdb_ff), Some(_)) => Err(SecondNotLowSurrogate),
            (Some(&0xd8_00...0xdb_ff), None) => Err(MissingSecond),
            (Some(_), _) => unreachable!()
        }}
    }

    fn from_utf16_tuple(utf16: (u16, Option<u16>)) -> Result<Self, InvalidUtf16Tuple> {
        use errors::InvalidUtf16Tuple::*;
        unsafe{ match utf16 {
            (0x00_00...0xd7_ff, None) | // single
            (0xe0_00...0xff_ff, None) | // single
            (0xd8_00...0xdb_ff, Some(0xdc_00...0xdf_ff)) // correct surrogate
                => Ok(char::from_utf16_tuple_unchecked(utf16)),
            (0xd8_00...0xdb_ff, Some(_)) => Err(InvalidSecond),
            (0xd8_00...0xdb_ff, None   ) => Err(MissingSecond),
            (0xdc_00...0xdf_ff,    _   ) => Err(FirstIsTrailingSurrogate),
            (        _        , Some(_)) => Err(SuperfluousSecond),
            (        _        , None   ) => unreachable!()
        }}
    }

    unsafe fn from_utf16_tuple_unchecked(utf16: (u16, Option<u16>)) -> Self {
        match utf16.1 {
            Some(second) => combine_surrogates(utf16.0, second),
            None         => char::from_u32_unchecked(utf16.0 as u32)
        }
    }


    fn from_u32_detailed(c: u32) -> Result<Self,InvalidCodepoint> {
        use errors::InvalidCodepoint::*;
        unsafe{ match c {
            0x00_00_00...0x00_d7_ff => Ok(char::from_u32_unchecked(c)),
            0x00_d8_00...0x00_df_ff => Err(Utf16Reserved),
            0x00_e0_00...0x10_ff_ff => Ok(char::from_u32_unchecked(c)),
            0x11_00_00...u32::MAX   => Err(TooHigh),
                       _            => unreachable!()
        }}
    }
}

// Adapted from https://www.cl.cam.ac.uk/~mgk25/ucs/utf8_check.c
fn overlong(first: u8, second: u8) -> bool {
    if first < 0x80 {
        false
    } else if (first & 0xe0) == 0xc0 {
        (first & 0xfe) == 0xc0
    } else if (first & 0xf0) == 0xe0 {
        first == 0xe0 && (second & 0xe0) == 0x80
    } else {
        first == 0xf0 && (second & 0xf0) == 0x80
    }
}

// Create a `char` from a leading and a trailing surrogate.
unsafe fn combine_surrogates(first: u16, second: u16) -> char {
    let high = (first & 0x_03_ff) as u32;
    let low = (second & 0x_03_ff) as u32;
    let c = ((high << 10) | low) + 0x_01_00_00; // no, the constant can't be or'd in
    char::from_u32_unchecked(c)
}



/// Adds `.utf8chars()` and `.utf16chars()` iterator constructors to `&str`.
pub trait StrExt: AsRef<str> {
    /// Equivalent to `.chars()` but produces `Utf8Char`s.
    fn utf8chars(&self) -> Utf8Chars;
    /// Equivalent to `.chars()` but produces `Utf16Char`s.
    fn utf16chars(&self) -> Utf16Chars;
    /// Equivalent to `.char_indices()` but produces `Utf8Char`s.
    fn utf8char_indices(&self) -> Utf8CharIndices;
    /// Equivalent to `.char_indices()` but produces `Utf16Char`s.
    fn utf16char_indices(&self) -> Utf16CharIndices;
}

impl StrExt for str {
    fn utf8chars(&self) -> Utf8Chars {
        Utf8Chars::from(self)
    }
    fn utf16chars(&self) -> Utf16Chars {
        Utf16Chars::from(self)
    }
    fn utf8char_indices(&self) -> Utf8CharIndices {
        Utf8CharIndices::from(self)
    }
    fn utf16char_indices(&self) -> Utf16CharIndices {
        Utf16CharIndices::from(self)
    }
}

#[cfg(feature="ascii")]
impl StrExt for AsciiStr {
    fn utf8chars(&self) -> Utf8Chars {
        Utf8Chars::from(self.as_str())
    }
    fn utf16chars(&self) -> Utf16Chars {
        Utf16Chars::from(self.as_str())
    }
    fn utf8char_indices(&self) -> Utf8CharIndices {
        Utf8CharIndices::from(self.as_str())
    }
    fn utf16char_indices(&self) -> Utf16CharIndices {
        Utf16CharIndices::from(self.as_str())
    }
}



/// Adds methods for splitting and merging `Utf8Char` and `Utf16Char` to and
/// from `u8`s or `u16`s.
pub trait IterExt: Iterator+Sized {
    /// Converts an iterator of `Utf8Char` or `&Utf8Char` to an iterator of 
    /// `u8`s.  
    /// Has the same effect as `.flat_map()` or `.flatten()`, but the returned
    /// iterator is ~40% faster.
    ///
    /// The iterator also implements `Read`
    /// (when the `std` feature isn't disabled).  
    /// Reading will never produce an error, and calls to `.read()` and `.next()`
    /// can be mixed.
    ///
    /// The exact number of bytes cannot be known in advance, but `size_hint()`
    /// gives the possible range.
    /// (min: all remaining characters are ASCII, max: all require four bytes)
    ///
    /// # Examples
    ///
    /// From iterator of values:
    ///
    /// ```
    /// use encode_unicode::{IterExt, StrExt};
    ///
    /// let iterator = "foo".utf8chars();
    /// let mut bytes = [0; 4];
    /// for (u,dst) in iterator.to_bytes().zip(&mut bytes) {*dst=u;}
    /// assert_eq!(&bytes, b"foo\0");
    /// ```
    ///
    /// From iterator of references:
    ///
    #[cfg_attr(feature="std", doc=" ```")]
    #[cfg_attr(not(feature="std"), doc=" ```no_compile")]
    /// use encode_unicode::{IterExt, StrExt, Utf8Char};
    ///
    /// let chars: Vec<Utf8Char> = "💣 bomb 💣".utf8chars().collect();
    /// let bytes: Vec<u8> = chars.iter().to_bytes().collect();
    /// let flat_map: Vec<u8> = chars.iter().flat_map(|u8c| *u8c ).collect();
    /// assert_eq!(bytes, flat_map);
    /// ```
    ///
    /// `Read`ing from it:
    ///
    #[cfg_attr(feature="std", doc=" ```")]
    #[cfg_attr(not(feature="std"), doc=" ```no_compile")]
    /// use encode_unicode::{IterExt, StrExt};
    /// use std::io::Read;
    ///
    /// let s = "Ååh‽";
    /// assert_eq!(s.len(), 8);
    /// let mut buf = [b'E'; 9];
    /// let mut reader = s.utf8chars().to_bytes();
    /// assert_eq!(reader.read(&mut buf[..]).unwrap(), 8);
    /// assert_eq!(reader.read(&mut buf[..]).unwrap(), 0);
    /// assert_eq!(&buf[..8], s.as_bytes());
    /// assert_eq!(buf[8], b'E');
    /// ```
    fn to_bytes(self) -> Utf8CharSplitter<Self::Item,Self> where Self::Item: Borrow<Utf8Char>;

    /// Converts an iterator of `Utf16Char` (or `&Utf16Char`) to an iterator of
    /// `u16`s.  
    /// Has the same effect as `.flat_map()` or `.flatten()`, but the returned
    /// iterator is about twice as fast.
    ///
    /// The exact number of units cannot be known in advance, but `size_hint()`
    /// gives the possible range.
    ///
    /// # Examples
    ///
    /// From iterator of values:
    ///
    /// ```
    /// use encode_unicode::{IterExt, StrExt};
    ///
    /// let iterator = "foo".utf16chars();
    /// let mut units = [0; 4];
    /// for (u,dst) in iterator.to_units().zip(&mut units) {*dst=u;}
    /// assert_eq!(units, ['f' as u16, 'o' as u16, 'o' as u16, 0]);
    /// ```
    ///
    /// From iterator of references:
    ///
    #[cfg_attr(feature="std", doc=" ```")]
    #[cfg_attr(not(feature="std"), doc=" ```no_compile")]
    /// use encode_unicode::{IterExt, StrExt, Utf16Char};
    ///
    /// // (💣 takes two units)
    /// let chars: Vec<Utf16Char> = "💣 bomb 💣".utf16chars().collect();
    /// let units: Vec<u16> = chars.iter().to_units().collect();
    /// let flat_map: Vec<u16> = chars.iter().flat_map(|u16c| *u16c ).collect();
    /// assert_eq!(units, flat_map);
    /// ```
    fn to_units(self) -> Utf16CharSplitter<Self::Item,Self> where Self::Item: Borrow<Utf16Char>;
}

impl<I:Iterator> IterExt for I {
    fn to_bytes(self) -> Utf8CharSplitter<Self::Item,Self> where Self::Item: Borrow<Utf8Char> {
        iter_bytes(self)
    }
    fn to_units(self) -> Utf16CharSplitter<Self::Item,Self> where Self::Item: Borrow<Utf16Char> {
        iter_units(self)
    }
}
