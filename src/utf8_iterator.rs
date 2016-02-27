/* Copyright 2016 Torbjørn Birch Moltu
 *
 * Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
 * http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
 * http://opensource.org/licenses/MIT>, at your option. This file may not be
 * copied, modified, or distributed except according to those terms.
 */

use Utf8Char;
extern crate std;
use std::{fmt,mem,u32,usize};
use std::ops::Not;
use std::io::{Read, Error as ioError};



/// Read or iterate over the bytes in an UTF-8 representation of a codepoint.
#[derive(Clone,Copy)]
pub struct Utf8Iterator (u64);

impl From<Utf8Char> for Utf8Iterator {
    fn from(uc: Utf8Char) -> Self {
        let bytes: u32 = unsafe{ mem::transmute(uc) };
        Utf8Iterator((usize::MAX << uc.len()*8) as u64   |  u32::from_le(bytes) as u64)
    }
}
impl From<char> for Utf8Iterator {
    fn from(c: char) -> Self {
        // call ucfromc, goto ucifromuc
        Self::from(Utf8Char::from(c))
    }
}
impl Iterator for Utf8Iterator {
    type Item=u8;
    fn next(&mut self) -> Option<u8> {
        let next = self.0 as u8;
        if next == 255 {
            None
        } else {
            self.0 >>= 8;
            Some(next)
        }
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        // call len, load stack a, push #1, push a, ret
        (self.len(),  Some(self.len()))
    }
}
impl ExactSizeIterator for Utf8Iterator {
    fn len(&self) -> usize {// not straightforward, but possible
        let has_read_bits = self.0.trailing_zeros();// number of shifts x 8
        let shifts_undone = self.0 << has_read_bits;
        let unused_bits = shifts_undone.not().trailing_zeros();
        let unused_bytes = (has_read_bits+unused_bits) / 8;
        // leading ones in the last content byte gets truncated when we divided by eigth.
        (8-unused_bytes) as usize
    }
}
impl Read for Utf8Iterator {
    /// Always returns Ok
    fn read(&mut self,  buf: &mut[u8]) -> Result<usize, ioError> {
        let mut wrote = 0;
        while let Some(ptr) = buf.get_mut(wrote) {// while loop because I need the counter afterwards
            if let Some(b) = self.next() {// don't call self.next() untill I know I can write it.
                *ptr = b;
                wrote += 1;
            } else {
                 break;
            }
        }
        Ok(wrote)
    }
}
impl fmt::Debug for Utf8Iterator {
    fn fmt(&self,  fmtr: &mut fmt::Formatter) -> fmt::Result {
        let content: Vec<u8> = self.collect();
        write!(fmtr, "bytes left: {:?}, content: {:x}", content, self.0)
    }
}
