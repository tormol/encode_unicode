/* Copyright 2016 Torbjørn Birch Moltu
 *
 * Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
 * http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
 * http://opensource.org/licenses/MIT>, at your option. This file may not be
 * copied, modified, or distributed except according to those terms.
 */

#![feature(unicode)]// reference implementations of to_slice

//! Test that every method gives the correct result for valid values.
//! Except iterators, which are stateful.

use std::char;
use std::str::{self,FromStr};
use std::hash::{Hash,SipHasher};
extern crate encode_unicode;
use encode_unicode::*;


#[test]
fn equal_defaults() {
    assert_eq!(Utf8Char::default().to_char(), char::default());
    assert_eq!(Utf16Char::default().to_char(), char::default());
}

#[test]
fn same_size_as_char() {
    use std::mem::size_of;
    assert_eq!(size_of::<Utf8Char>(), size_of::<char>());
    assert_eq!(size_of::<Utf16Char>(), size_of::<char>());
}


const EDGES_AND_BETWEEN: [u32;13] = [
    0x0,// min
    0x3b,// between
    0x7f,// max 1-byte UTF-8
    0x80,// min 2-byte UTF-8
    0x111,// between
    0x7ff,// max 2-byte UTF-8
    0x800,// min 3-byte UTF-8
    0xd7ff,// before reserved
    0xe000,// after reserved
    0xffff,// max UTF-16 single and 3-byte UTF-8
    0x10000,// min UTF-16 surrogate and 4-byte UTF-8
    0xabcde,// between
    0x10ffff,// max
];


fn test(c: u32) {
    let c = char::from_u32(c).expect(&format!("{:x} is not a valid char", c));
    assert_eq!(char::from_u32_detailed(c as u32), Ok(c));
    let sh = &mut SipHasher::new();

    // UTF-8
    let uc = c.to_utf8();
    assert_eq!(uc.to_char(), c);
    assert_eq!(uc.hash(sh), c.hash(sh));

    let mut reference_dst = [0;4];
    let mut len = None;
    for i in 0..5 {
        let mut test_dst = [0;4];
        len = c.encode_utf8(&mut reference_dst[..i]);
        assert_eq!(c.to_utf8_slice(&mut test_dst[..i]), len);
        assert_eq!(test_dst, reference_dst);
        assert_eq!(uc.to_slice(&mut test_dst[..i]), len);
        assert_eq!(test_dst, reference_dst);
    }
    let len = len.expect(&format!("encode_utf8 never succeded: c={}={:x}, utf8={:?}", c, c as u32, reference_dst));
    let str_ = str::from_utf8(&reference_dst[..len]).unwrap();
    let ustr = Utf8Char::from_str(str_).unwrap();
    assert_eq!(ustr.to_array().0, uc.to_array().0);// bitwise equality

    assert_eq!(reference_dst[0].extra_utf8_bytes(), Ok(len-1));
    assert_eq!(reference_dst[0].extra_utf8_bytes_unchecked(), len-1);
    assert_eq!(c.to_utf8_array(),  (reference_dst, len));
    assert_eq!(char::from_utf8_array(reference_dst), Ok(c));
    assert_eq!(char::from_utf8_slice(&reference_dst[..len]), Ok((c,len)));
    for other in &EDGES_AND_BETWEEN {
        let other = unsafe{ char::from_u32_unchecked(*other) };
        let uother = other.to_utf8();
        assert_eq!(uc == uother,  c == other);
        assert_eq!(uc.hash(sh)==other.hash(sh),  c.hash(sh)==uother.hash(sh));
        assert_eq!(uc.cmp(&uother), c.cmp(&other));
    }
    assert_eq!(uc.to_array(),  (reference_dst, len));
    assert_eq!(Utf8Char::from_array(reference_dst), Ok(uc));
    assert_eq!(Utf8Char::from_slice_start(&reference_dst[..len]), Ok((uc,len)));
    let iterated: Vec<_> = c.iter_utf8_bytes().collect();
    assert_eq!(iterated[..], reference_dst[..len]);
    assert_eq!(<AsRef<[u8]>>::as_ref(&uc), &iterated[..]);

    // UTF-16
    let uc = c.to_utf16();
    assert_eq!(uc.to_char(), c);
    let mut reference_dst = [0;2];
    let mut len = None;
    for i in 0..3 {
        let mut test_dst = [0;2];
        len = c.encode_utf16(&mut reference_dst[..i]);
        assert_eq!(c.to_utf16_slice(&mut test_dst[..i]), len);
        assert_eq!(test_dst, reference_dst);
        assert_eq!(uc.to_slice(&mut test_dst[..i]), len);
        assert_eq!(test_dst, reference_dst);
    }
    let len = len.unwrap();
    assert_eq!(reference_dst[0].utf16_needs_extra_unit(), Some(len==2));
    assert_eq!(reference_dst[0].utf16_is_leading_surrogate(), len==2);
    let tuple = c.to_utf16_tuple();
    assert_eq!([tuple.0, tuple.1.unwrap_or(0)],  reference_dst);
    assert_eq!(char::from_utf16_tuple(tuple), Ok(c));
    assert_eq!(c.to_utf16().to_char(), c);
    let iterated: Vec<_> = c.iter_utf16_units().collect();
    assert_eq!(*iterated, reference_dst[0..len]);
    assert_eq!(<AsRef<[u16]>>::as_ref(&uc), &iterated[..]);
}


#[test]
fn edges_middle() {
    for c in &EDGES_AND_BETWEEN {
        test(*c);
    }
}


#[test]
#[ignore]
fn all() {
    for c in std::iter::Iterator::chain(0..0xd800, 0xe000..0x110000) {
        test(c);
    }
}
