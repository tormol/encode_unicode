/* Copyright 2016 Torbjørn Birch Moltu
 *
 * Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
 * http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
 * http://opensource.org/licenses/MIT>, at your option. This file may not be
 * copied, modified, or distributed except according to those terms.
 */

//! iterators are stateful; test that they behave.

use std::io::Read;
extern crate encode_unicode;
use encode_unicode::CharExt;
//use encode_unicode::{Utf8Iterator,Utf16Iterator};

#[test]
fn read_single_byte() {
    let mut buf = [0; 4];
    for c in 0..128 {
        let uc = char::from_u32_detailed(c).unwrap().to_utf8();
        let mut iter = uc.into_iter();
        assert_eq!((iter.read(&mut buf[..4]).unwrap(),c), (1,c));
        assert_eq!((iter.read(&mut buf[..4]).unwrap(),c), (0,c));
        let mut iter = uc.into_iter();
        assert_eq!((iter.read(&mut buf[..1]).unwrap(),c), (1,c));
        assert_eq!((iter.read(&mut buf[..1]).unwrap(),c), (0,c));
        let mut iter = uc.into_iter();
        assert_eq!((iter.read(&mut buf[..0]).unwrap(),c), (0,c));
        assert_eq!((iter.read(&mut buf[..2]).unwrap(),c), (1,c));
    }
}
