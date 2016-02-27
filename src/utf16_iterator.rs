/* Copyright 2016 Torbjørn Birch Moltu
 *
 * Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
 * http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
 * http://opensource.org/licenses/MIT>, at your option. This file may not be
 * copied, modified, or distributed except according to those terms.
 */

use CharExt;
use Utf16Char;
extern crate std;
use std::fmt;


/// Iterate over the units in an UTF-16 representation of a codepoint.
#[derive(Clone,Copy)]
pub struct Utf16Iterator {
    first: Option<u16>,
    second: Option<u16>,
}
impl From<char> for Utf16Iterator {
    fn from(c: char) -> Self {
        let (first, second) = c.to_utf16_tuple();
        Utf16Iterator{ first: Some(first),  second: second }
    }
}
impl From<Utf16Char> for Utf16Iterator {
    fn from(uc: Utf16Char) -> Self {
        let (first, second) = uc.to_tuple();
        Utf16Iterator{ first: Some(first),  second: second }
    }
}
impl Iterator for Utf16Iterator {
    type Item=u16;
    fn next(&mut self) -> Option<u16> {
        match (self.first, self.second) {
            (Some(first), _)     => {self.first = None;   Some(first) },
            (None, Some(second)) => {self.second = None;  Some(second)},
            (None, None)         => {                     None        },
        }
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len(), Some(self.len()))
    }
}
impl ExactSizeIterator for Utf16Iterator {
    fn len(&self) -> usize {
        match (self.first, self.second) {
            (None   ,  None   )  =>  0,
            (Some(_),  None   )  =>  1,
            (None   ,  Some(_))  =>  1,
            (Some(_),  Some(_))  =>  2,
        }
    }
}
impl fmt::Debug for Utf16Iterator {
    fn fmt(&self,  fmtr: &mut fmt::Formatter) -> fmt::Result {
        let v: Vec<u16> = self.collect();
        write!(fmtr, "{:?}", v)
    }
}
