/* Copyright 2016 Torbjørn Birch Moltu
 *
 * Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
 * http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
 * http://opensource.org/licenses/MIT>, at your option. This file may not be
 * copied, modified, or distributed except according to those terms.
 */

use Utf8Char;
extern crate core;
use self::core::{mem, u32, u64};
use self::core::ops::Not;
#[cfg(not(feature="no_std"))]
use self::core::fmt;
#[cfg(not(feature="no_std"))]
use std::io::{Read, Error as ioError};



/// Read or iterate over the bytes in the UTF-8 representation of a codepoint.
#[derive(Clone,Copy)]
pub struct Utf8Iterator (u32);

impl From<Utf8Char> for Utf8Iterator {
    fn from(uc: Utf8Char) -> Self {
        let used = u32::from_le(unsafe{ mem::transmute(uc) });
        // uses u64 because shifting an u32 by 32 bits is a no-op.
        let unused_set = (u64::MAX  <<  uc.len() as u64*8) as u32;
        Utf8Iterator(used | unused_set)
    }
}
impl From<char> for Utf8Iterator {
    fn from(c: char) -> Self {
        Self::from(Utf8Char::from(c))
    }
}
impl Iterator for Utf8Iterator {
    type Item=u8;
    fn next(&mut self) -> Option<u8> {
        let next = self.0 as u8;
        if next == 0xff {
            None
        } else {
            self.0 = (self.0 >> 8)  |  0xff_00_00_00;
            Some(next)
        }
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len(),  Some(self.len()))
    }
}
impl ExactSizeIterator for Utf8Iterator {
    fn len(&self) -> usize {// not straightforward, but possible
        let unused_bytes = self.0.not().leading_zeros() / 8;
        4 - unused_bytes as usize
    }
}
#[cfg(not(feature="no_std"))]
impl Read for Utf8Iterator {
    /// Always returns Ok
    fn read(&mut self,  buf: &mut[u8]) -> Result<usize, ioError> {
        // Cannot call self.next() until I know I can write the result.
        for (i, dst) in buf.iter_mut().enumerate() {
            match self.next() {
                Some(b) => *dst = b,
                None    => return Ok(i),
            }
        }
        Ok(buf.len())
    }
}
#[cfg(not(feature="no_std"))]
impl fmt::Debug for Utf8Iterator {
    fn fmt(&self,  fmtr: &mut fmt::Formatter) -> fmt::Result {
        let content: Vec<u8> = self.collect();
        write!(fmtr, "bytes left: {:?}, content: {:x}", content, self.0)
    }
}
