/* Copyright 2016 Torbjørn Birch Moltu
 *
 * Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
 * http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
 * http://opensource.org/licenses/MIT>, at your option. This file may not be
 * copied, modified, or distributed except according to those terms.
 */

//! Test that every method gives the correct result for valid values.
//! Except iterators, which are stateful.

use std::char;
use std::str::{self,FromStr};
use std::hash::Hash;
use std::collections::hash_map::DefaultHasher;
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

#[test]
#[cfg(feature="std")]
fn read_iterator() {
    use std::io::Read;
    use std::cmp::min;

    let uc = 'ä'.to_utf8();
    assert_eq!(uc.len(), 2);
    for chunk in 1..5 {
        let mut buf = [b'E'; 6];
        let mut iter = uc.into_iter();
        let mut written = 0;
        for _ in 0..4 {
            assert_eq!(iter.read(&mut buf[..0]).unwrap(), 0);
            let wrote = iter.read(&mut buf[written..written+chunk]).unwrap();
            assert_eq!(wrote, min(2-written, chunk));
            written += wrote;
            for &b in &buf[written..] {assert_eq!(b, b'E');}
            assert_eq!(buf[..written], AsRef::<[u8]>::as_ref(&uc)[..written]);
        }
        assert_eq!(written, 2);
    }
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

fn eq_cmp_hash(c: char) -> (Utf8Char, Utf16Char) {
    let sh = &mut DefaultHasher::new();
    let u8c = c.to_utf8();
    assert_eq!(u8c.to_char(), c);
    assert_eq!(u8c.hash(sh), c.hash(sh));
    let u16c = c.to_utf16();
    assert_eq!(u16c.to_char(), c);
    assert_eq!(u16c.hash(sh), c.hash(sh));

    for other in &EDGES_AND_BETWEEN {
        let other = unsafe{ char::from_u32_unchecked(*other) };

        let u8other = other.to_utf8();
        assert_eq!(u8c == u8other,  c == other);
        assert_eq!(u8c.hash(sh)==other.hash(sh),  c.hash(sh)==u8other.hash(sh));
        assert_eq!(u8c.cmp(&u8other), c.cmp(&other));

        let u16other = other.to_utf16();
        assert_eq!(u16c == u16other,  c == other);
        assert_eq!(u16c.hash(sh)==other.hash(sh),  c.hash(sh)==u16other.hash(sh));
        assert_eq!(u16c.cmp(&u16other), c.cmp(&other));
    }
    (u8c, u16c)
}

fn iterators(c: char) {
    let mut iter = c.iter_utf8_bytes();
    let mut buf = [0; 4];
    let mut iter_ref = c.encode_utf8(&mut buf[..]).as_bytes().iter();
    for _ in 0..6 {
        assert_eq!(iter.size_hint(), iter_ref.size_hint());
        assert_eq!(format!("{:?}", iter), format!("{:?}", iter_ref.as_slice()));
        assert_eq!(iter.next(), iter_ref.next().cloned());
    }

    let mut iter = c.iter_utf16_units();
    let mut buf = [0; 2];
    let mut iter_ref = c.encode_utf16(&mut buf[..]).iter();
    for _ in 0..4 {
        assert_eq!(iter.size_hint(), iter_ref.size_hint());
        assert_eq!(format!("{:?}", iter), format!("{:?}", iter_ref.as_slice()));
        assert_eq!(iter.next(), iter_ref.next().cloned());
    }
}

fn test(c: u32) {
    let c = char::from_u32(c).expect(&format!("{:x} is not a valid char", c));
    assert_eq!(char::from_u32_detailed(c as u32), Ok(c));
    let (u8c, u16c) = eq_cmp_hash(c);
    iterators(c);
    assert_eq!(Utf16Char::from(u8c), u16c);
    assert_eq!(Utf8Char::from(u16c), u8c);

    // UTF-8
    let mut buf = [0; 4];
    let reference = c.encode_utf8(&mut buf[..]).as_bytes();
    let len = reference.len(); // short name because it is used in many places.
    assert_eq!(reference[0].extra_utf8_bytes(), Ok(len-1));
    assert_eq!(reference[0].extra_utf8_bytes_unchecked(), len-1);
    assert_eq!(AsRef::<[u8]>::as_ref(&u8c), reference);

    let (mut arr,arrlen) = u8c.to_array();
    assert_eq!(arrlen, len);
    assert_eq!(Utf8Char::from_array(arr), Ok(u8c));
    assert_eq!(c.to_utf8_array(),  (arr, len));

    let str_ = str::from_utf8(reference).unwrap();
    let ustr = Utf8Char::from_str(str_).unwrap();
    assert_eq!(ustr.to_array().0, arr);// bitwise equality
    assert_eq!(char::from_utf8_array(arr), Ok(c));
    assert_eq!(char::from_utf8_slice(reference), Ok((c,len)));
    for b in arr.iter_mut().skip(arrlen) {
        *b = b'F';// from_slice_start must not read these.
    }
    assert_eq!(Utf8Char::from_slice_start(&arr), Ok((u8c,len)));// Test that it doesn't read too much
    assert_eq!(Utf8Char::from_slice_start(reference), Ok((u8c,len)));

    // UTF-16
    let mut buf = [0; 2];
    let reference = c.encode_utf16(&mut buf[..]);
    let len = reference.len();
    assert_eq!(reference[0].utf16_needs_extra_unit(), Ok(len==2));
    assert_eq!(reference[0].utf16_is_leading_surrogate(), len==2);
    assert_eq!(u16c.as_ref(), reference);
    assert_eq!(char::from_utf16_slice(&reference[..len]), Ok((c,len)));
    let tuple = c.to_utf16_tuple();
    assert_eq!(tuple, (reference[0],reference.get(1).cloned()));
    assert_eq!(char::from_utf16_tuple(tuple), Ok(c));
    assert_eq!(c.to_utf16().to_char(), c);
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
