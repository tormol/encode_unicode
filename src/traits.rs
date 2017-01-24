/* Copyright 2016 Torbjørn Birch Moltu
 *
 * Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
 * http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
 * http://opensource.org/licenses/MIT>, at your option. This file may not be
 * copied, modified, or distributed except according to those terms.
 */

#![allow(unused_unsafe)]// explicit unsafe{} blocks in unsafe functions are a good thing.

use Utf8Char;
use Utf16Char;
use Utf8Iterator;
use Utf16Iterator;
use error::*;
extern crate core;
use self::core::{char, u32, mem};
use self::core::ops::Not;

// TODO better docs and tests

/// Methods for working with `u8`s as UTF-8 bytes.
pub trait U8UtfExt {
    /// How many more bytes will you need to complete this codepoint?
    ///
    /// # Errors
    /// An error is returned if this is not a valid start of an UTF-8 codepoint:
    /// * `128..192`: ContinuationByte
    /// * `240..`: TooLongSequence
    fn extra_utf8_bytes(self) -> Result<usize,InvalidUtf8FirstByte>;

    /// How many more bytes will you need to complete this codepoint?
    ///
    /// This function assumes that self is a valid UTF-8 start,
    /// and will return gibberish otherwize.
    /// The formula is `self.not().leading_zeros().saturating_sub(1)`.
    fn extra_utf8_bytes_unchecked(self) -> usize;
}

impl U8UtfExt for u8 {
    fn extra_utf8_bytes(self) -> Result<usize,InvalidUtf8FirstByte> {
        use error::InvalidUtf8FirstByte::{ContinuationByte,TooLongSeqence};
        match self.not().leading_zeros() {
            1       =>  Err(ContinuationByte),// following byte
            5...255 =>  Err(TooLongSeqence),// too big
            0       =>  Ok(0),// ASCII
            n       =>  Ok(n as usize-1),// start of multibyte
        }
    }
    fn extra_utf8_bytes_unchecked(self) -> usize {
        (self.not().leading_zeros()as usize).saturating_sub(1)
    }
}


/// Methods for working with `u16`s as UTF-16 units.
pub trait U16UtfExt {
    /// Will you need an extra unit to complete this codepoint?
    ///
    /// # Failures:
    ///
    /// 0xdc00..0xe000
    fn utf16_needs_extra_unit(self) -> Option<bool>;

    /// Does this `u16` need another `u16` to complete a codepoint?
    /// Returns `(self & 0xfc00) == 0xd800`
    fn utf16_is_leading_surrogate(self) -> bool;
}
impl U16UtfExt for u16 {
    fn utf16_needs_extra_unit(self) -> Option<bool> {match self {
        // https://en.wikipedia.org/wiki/UTF-16#U.2B10000_to_U.2B10FFFF
        0x_dc_00...0x_df_ff => None,
        0x_d8_00...0x_db_ff => Some(true),
        _                   => Some(false),
    }}

    fn utf16_is_leading_surrogate(self) -> bool {
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
    fn from_utf8_slice(src: &[u8]) -> Result<(Self,usize),InvalidUtf8Slice>;

    /// Create a `char` from the start of a UTF-16 slice,
    /// and also return how many units were used.
    fn from_utf16_slice(src: &[u16]) -> Result<(Self,usize), InvalidUtf16Slice>;


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
            // Now, unused bytes are zero, (which matters for Utf8Char.eq()) and the rest are 0b10xx_xxxx

            // set header on first byte
            parts |= (0xff_00u32 >> len)  &  0xff;// store length
            parts &= Not::not(1u32 << 7-len);// clear the next bit after it

            let bytes: [u8; 4] = unsafe{ mem::transmute(u32::from_le(parts)) };
            (bytes, len)
        }
    }


    fn from_utf8_slice(src: &[u8]) -> Result<(Self,usize),InvalidUtf8Slice> {
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
                c = (c << 6)  |  (b & 0b00111111) as u32;
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


    fn from_utf16_slice(src: &[u16]) -> Result<(Self,usize), InvalidUtf16Slice> {
        use errors::InvalidUtf16Slice::*;
        unsafe {match (src.get(0), src.get(1)) {
            (Some(&u @ 0x00_00...0xd7_ff), _) |
            (Some(&u @ 0xde_00...0xff_ff), _)
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


// If all the data bits in the first byte are zero, the sequence might be longer than necessary
// When you go up one byte, you gain 6-1 data bits, so if the five first are zero it's too long.
// The first byte has 3 + (4-len) data bits, which we know are zero.
// The first two bits in the second byte are 10, which gets shifted out.
fn overlong(first: u8,  second: u8) -> bool {
    let both = ((first as u16) << 8)  |  (second << 2) as u16;
    let both = both << 1+both.not().leading_zeros();
    both.leading_zeros() >= 5
}

// Create a `char` from a leading and a trailing surrogate.
unsafe fn combine_surrogates(first: u16, second: u16) -> char {
    let high = (first & 0x_03_ff) as u32;
    let low = (second & 0x_03_ff) as u32;
    let c = ((high << 10) | low) + 0x_01_00_00; // no, the constant can't be or'd in
    char::from_u32_unchecked(c)
}
