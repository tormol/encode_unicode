/* Copyright 2016 Torbjørn Birch Moltu
 *
 * Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
 * http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
 * http://opensource.org/licenses/MIT>, at your option. This file may not be
 * copied, modified, or distributed except according to those terms.
 */

//! Test that methods gives the correct error.
//! Some also test a bit more because it's easy.

mod codepoint_iterators;
use codepoint_iterators::*;
use std::char;
extern crate encode_unicode;
use encode_unicode::*;
use encode_unicode::error::*;


#[test]
fn from_u32() {
    use encode_unicode::error::InvalidCodePoint::*;
    for c in iter_some_invalid() {
        assert_eq!(char::from_u32_detailed(c).ok(), char::from_u32(c));
    }

    // test edges and somewhere in between
    assert_eq!(char::from_u32_detailed(0xd800), Err(Utf16Reserved));
    assert_eq!(char::from_u32_detailed(0xdcba), Err(Utf16Reserved));
    assert_eq!(char::from_u32_detailed(0xdfff), Err(Utf16Reserved));
    assert_eq!(char::from_u32_detailed(0x110000), Err(TooHigh));
    assert_eq!(char::from_u32_detailed(0xabcdef), Err(TooHigh));
    assert_eq!(char::from_u32_detailed(0xffffff), Err(TooHigh));
}

#[test]
fn utf8_extra_bytes() {
    use encode_unicode::error::InvalidUtf8FirstByte::*;
    for c in 0..256 {
        assert_eq!( (c as u8).extra_utf8_bytes(), match c {
            0b_1000_0000...0b_1011_1111 => Err(ContinuationByte),
            0b_1111_1000...0b_1111_1111 => Err(TooLongSeqence),
            0b_0000_0000...0b_0111_1111 => Ok(0),
            0b_1100_0000...0b_1101_1111 => Ok(1),
            0b_1110_0000...0b_1110_1111 => Ok(2),
            0b_1111_0000...0b_1111_0111 => Ok(3),
                         _              => unreachable!(),
        });
    }
}

#[test]
fn utf16_extra_unit() {
    for c in 0..0x1_00_00 {
        assert_eq!( (c as u16).utf16_needs_extra_unit(), match c {
            0b_0000_0000_0000_0000...0b_1101_0111_1111_1111 => Some(false),
            0b_1101_1000_0000_0000...0b_1101_1011_1111_1111 => Some(true),
            0b_1101_1100_0000_0000...0b_1101_1111_1111_1111 => None,
            0b_1110_0000_0000_0000...0b_1111_1111_1111_1111 => Some(false),
                                   _                        => unreachable!(),
        });
    }
}


#[test]
fn from_utf16_tuple() {
	use encode_unicode::error::InvalidUtf16Tuple::*;
	assert_eq!(char::from_utf16_tuple((0xdcba,Some(0))), Err(FirstIsTrailingSurrogate));
}

#[test]
fn overlong() {
    use encode_unicode::error::InvalidUtf8::OverLong;
    let overlongs = [[0xc0,0xbf], [0xe0,0x9f], [0xf0,0x8f],
                     [0xc0,0x9f], [0xe0,0x8f], [0xf0,0x87]];
    for o in overlongs.iter() {
        let arr = [o[0],o[1], 0x80, 0x80];
        assert_eq!(char::from_utf8_slice(&arr), Err(InvalidUtf8Slice::Utf8(OverLong)));
        assert_eq!(char::from_utf8_array(arr), Err(InvalidUtf8Array::Utf8(OverLong)));
    }
}

#[test]
fn from_str() {
    use std::str::FromStr;
    use encode_unicode::error::FromStrError::*;
    assert_eq!(Utf8Char::from_str(""), Err(Empty));
    assert_eq!(Utf8Char::from_str("ab"), Err(SeveralCodePoints));
    assert_eq!(Utf8Char::from_str("́e"), Err(SeveralCodePoints));// 'e'+u301 combining mark
}
