/* Copyright 2016 Torbjørn Birch Moltu
 *
 * Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
 * http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
 * http://opensource.org/licenses/MIT>, at your option. This file may not be
 * copied, modified, or distributed except according to those terms.
 */

use CharExt;
use Utf16Char;
#[cfg(feature="std")]
extern crate core;
#[cfg(feature="std")]
use self::core::fmt;

// Invalid values that says the field consumed or empty.
const FIRST_USED: u16 = 0x_dc_00;
const SECOND_USED: u16 = 0;

/// Iterate over the units in an UTF-16 representation of a codepoint.
#[derive(Clone,Copy)]
pub struct Utf16Iterator {
    first: u16,
    second: u16,
}
impl From<char> for Utf16Iterator {
    fn from(c: char) -> Self {
        let (first, second) = c.to_utf16_tuple();
        Utf16Iterator{ first: first,  second: second.unwrap_or(SECOND_USED) }
    }
}
impl From<Utf16Char> for Utf16Iterator {
    fn from(uc: Utf16Char) -> Self {
        let (first, second) = uc.to_tuple();
        Utf16Iterator{ first: first,  second: second.unwrap_or(SECOND_USED) }
    }
}
impl Iterator for Utf16Iterator {
    type Item=u16;
    fn next(&mut self) -> Option<u16> {
        match (self.first, self.second) {
            (FIRST_USED, SECOND_USED)  =>  {                            None        },
            (FIRST_USED, second     )  =>  {self.second = SECOND_USED;  Some(second)},
            (first     ,      _     )  =>  {self.first = FIRST_USED;    Some(first )},
        }
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len(), Some(self.len()))
    }
}
impl ExactSizeIterator for Utf16Iterator {
    fn len(&self) -> usize {
        (if self.first == FIRST_USED {0} else {1}) +
        (if self.second == SECOND_USED {0} else {1})
    }
}
#[cfg(feature="std")]
impl fmt::Debug for Utf16Iterator {
    fn fmt(&self,  fmtr: &mut fmt::Formatter) -> fmt::Result {
        let v: Vec<u16> = self.collect();
        write!(fmtr, "{:?}", v)
    }
}
