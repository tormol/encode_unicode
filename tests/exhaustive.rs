/* Copyright 2018 The encode_unicode Developers
 *
 * Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
 * http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
 * http://opensource.org/licenses/MIT>, at your option. This file may not be
 * copied, modified, or distributed except according to those terms.
 */

//! Tests that try all possible values for at least one parameter / byte / unit
//! of the tested function.

use std::char;
extern crate encode_unicode;
use encode_unicode::*;
use encode_unicode::error::*;
use encode_unicode::error::InvalidUtf8Array::*;
use encode_unicode::error::InvalidUtf8::*;
use encode_unicode::error::InvalidCodepoint::*;
use encode_unicode::error::InvalidUtf8FirstByte::*;

#[test]
fn from_ascii() {
    for cp in 0u32..256 {
        assert_eq!(Utf8Char::from_ascii(cp as u8).is_ok(), cp & 0x80 == 0);
        if let Ok(u8c) = Utf8Char::from_ascii(cp as u8) {
            assert_eq!(u8c, Utf8Char::from(cp as u8 as char));
        }
    }
}

#[test]
fn from_bmp() {
    for cp in 0u32..0x1_00_00 {
        assert_eq!(
            Utf16Char::from_bmp(cp as u16).ok(),
            char::from_u32(cp).map(|u32c| Utf16Char::from(u32c) )
        );
    }
}

#[test]
fn utf8_from_bytes() {
    for first in 0..256 {
        let array = [first as u8, 0b10_000000, 0b10_000000, 0b10_000000];
        let is_good = [0..128, 0xc2..0xe0, 0xe1..0xf0, 0xf1..0xf5].iter()
            // 0xe0 and 0xf0 are skipped because they currently create overlong values;
            // Setting the most significant data bit of the second byte also fixes that,
            // but creates UTF-16 reserved or too high codepoints.
            // TODO vary the second byte.
            .any(|range| first >= range.start  &&  first < range.end );
        let result = Utf8Char::from_array(array);
        assert_eq!(result.is_ok(), is_good, "first: {:#08b} ≈ {}", first, first as u8 as char);
        assert_eq!(char::from_utf8_array(array).is_ok(), is_good);
        assert_eq!(char::from_utf8_slice_start(&array).is_ok(), is_good);
        assert_eq!(Utf8Char::from_slice_start(&array).is_ok(), is_good);
        if let Ok(u8c) = result {
            assert!(u8c.to_array().0[u8c.len()..].iter().all(|&b| b == 0 ));
            for i in 1..u8c.len() {
                let mut corrupted = array;
                for &bad in &[0u8, 0x3f, 0x40, 0x7f, 0xc0, 0xcf] {
                    corrupted[i] = bad;
                    assert_eq!(
                        Utf8Char::from_array(corrupted),
                        Err(InvalidUtf8Array::Utf8(InvalidUtf8::NotAContinuationByte(i)))
                    );
                    assert_eq!(
                        char::from_utf8_array(corrupted),
                        Err(InvalidUtf8Array::Utf8(InvalidUtf8::NotAContinuationByte(i)))
                    );
                    assert_eq!(
                        Utf8Char::from_slice_start(&corrupted),
                        Err(InvalidUtf8Slice::Utf8(InvalidUtf8::NotAContinuationByte(i)))
                    );
                    assert_eq!(
                        char::from_utf8_slice_start(&corrupted),
                        Err(InvalidUtf8Slice::Utf8(InvalidUtf8::NotAContinuationByte(i)))
                    );
                }
            }
        } else if first > 0b1111_0111 {
            assert_eq!(result, Err(Utf8(FirstByte(TooLongSeqence))));
        } else if first > 0b1111_0100 {
            assert_eq!(result, Err(Codepoint(TooHigh)));
        } else if first > 0b10_111111 {
            assert_eq!(result, Err(Utf8(OverLong)), "{:#08b}", first);
        } else {
            assert_eq!(result, Err(Utf8(FirstByte(ContinuationByte))));
        }
    }
}
