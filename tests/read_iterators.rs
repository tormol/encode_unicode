/* Copyright 2018 Torbjørn Birch Moltu
 *
 * Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
 * http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
 * http://opensource.org/licenses/MIT>, at your option. This file may not be
 * copied, modified, or distributed except according to those terms.
 */

//! Tests for ensuring that iterators which also implement Read support
//! interleaving calls of `read()` and `next()`, and that they implement Read
//! correctly (support any buffer size at any time).

#![cfg(feature="std")]

use std::io::Read;
use std::cmp::min;
extern crate encode_unicode;
use encode_unicode::CharExt;
use encode_unicode::iterator::Utf8CharSplitter;

#[test]
fn read_single_ascii() {
    let uc = 'a'.to_utf8();
    assert_eq!(uc.len(), 1);
    for chunk in 1..5 {
        let mut buf = [b'E'; 6];
        let mut iter = uc.into_iter();
        let mut written = 0;
        for _ in 0..4 {
            assert_eq!(iter.read(&mut buf[..0]).unwrap(), 0);
            let wrote = iter.read(&mut buf[written..written+chunk]).unwrap();
            assert_eq!(wrote, min(1-written, chunk));
            written += wrote;
            for &b in &buf[written..] {assert_eq!(b, b'E');}
            assert_eq!(buf[..written], AsRef::<[u8]>::as_ref(&uc)[..written]);
        }
        assert_eq!(written, 1);
    }
}

#[test]
fn read_single_nonascii() {
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


#[test]
fn utf8charsplitter_read_all_sizes() {
    let s = "1111\u{104444}\u{222}1\u{833}1111\u{100004}";
    assert!(s.len()%3 == 1);
    let mut buf = vec![b'E'; s.len()+6];
    for size in 2..6 {//s.len()+4 {
        let mut reader = Utf8CharSplitter::from(s.chars().map(|c| c.to_utf8() ));
        for (offset, part) in s.as_bytes().chunks(size).enumerate() {
            let read_to = if part.len() == size {(offset+1)*size} else {buf.len()};
            assert_eq!(reader.read(&mut buf[offset*size..read_to]).unwrap(), part.len());
            assert_eq!(&buf[..offset*size+part.len()], &s.as_bytes()[..offset*size+part.len()]);
        }
        assert_eq!(reader.read(&mut buf[..]).unwrap(), 0);
        assert!(buf[s.len()..].iter().all(|&b| b==b'E' ));
    }
}

#[test]
fn utf8charsplitter_alternate_iter_read() {
    let s = "1111\u{104444}\u{222}1\u{833}1111\u{100004}";
    let mut buf = [b'0'; 10];
    for n in 0..2 {
        // need to collect to test size_hint()
        // because chars().size_hint() returns ((bytes+3)/4, Some(bytes))
        let u8chars = s.chars().map(|c| c.to_utf8() ).collect::<Vec<_>>();
        let mut iter: Utf8CharSplitter<_,_> = u8chars.into_iter().into();
        for (i, byte) in s.bytes().enumerate() {
            let until_next = s.as_bytes()[i..].iter().take_while(|&b| (b>>6)==0b10u8 ).count();
            let remaining_chars = s[i+until_next..].chars().count();
            println!("{}. run: byte {:02} of {}, remaining: {:02}+{}: 0b{:08b} = {:?}",
                     n, i, s.len(), remaining_chars, until_next, byte, byte as char);
            assert_eq!(iter.read(&mut[][..]).unwrap(), 0);
            if i % 2 == n {
                assert_eq!(iter.next(), Some(byte));
            } else {
                assert_eq!(iter.read(&mut buf[..1]).unwrap(), 1);
                assert_eq!(buf[0], byte);
            }
        }
        assert_eq!(iter.size_hint(), (0, Some(0)));
        assert_eq!(iter.next(), None);
        assert_eq!(iter.read(&mut buf[..]).unwrap(), 0);
    }
}
