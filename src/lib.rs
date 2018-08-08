/* Copyright 2016 Torbjørn Birch Moltu
 *
 * Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
 * http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
 * http://opensource.org/licenses/MIT>, at your option. This file may not be
 * copied, modified, or distributed except according to those terms.
 */


/*!
Miscellaneous UTF-8 and UTF-16 types and methods.

# Optional features:
* `#![no_std]`-mode: There are a few differences:
  * `AsciiExt` doesn't exist, but `is_ascii()` is made available as an inherent impl.
  * `Error` doesn't exist, but `description()` is made available as an inherent impl.
  * `Extend`/`FromIterator`-implementations for `String`/`Vec<u8>`/`Vec<u16>` are missing.
  * There is no `io`, so `Utf8Iterator` doesn't implement `Read`.

  This feature is enabled by setting `default-features=false` in `Cargo.toml`:
  `encode_unicode = {version="0.3", default-features=false}`
* Integration with the [ascii](https://tomprogrammer.github.io/rust-ascii/ascii/index.html) crate:  
  Convert `Utf8Char` and `Utf16Char` to and from
  [ascii::`AsciiChar`](https://tomprogrammer.github.io/rust-ascii/ascii/enum.AsciiChar.html).

The minimum supported version of Rust is 1.15,
older versions might work now but can break with a minor update.

[crates.io page](https://crates.io/crates/encode_unicode)  
[github repository](https://github.com/tormol/encode_unicode)

*/

#![warn(missing_docs)]

#![cfg_attr(not(feature="std"), no_std)]
// either `cargo clippy` doesn't see theese, or I get a warning when I build.
#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]
#![cfg_attr(feature="clippy", allow(derive_hash_xor_eq))]// tested
#![cfg_attr(feature="clippy", allow(len_without_is_empty))]// UtfxChar is never empty
#![cfg_attr(feature="clippy", allow(match_same_arms))]// looks better IMO
#![cfg_attr(feature="clippy", allow(needless_return))]// `foo.bar(); foo` looks unfinished
#![cfg_attr(feature="clippy", allow(redundant_closure))]// keep it explicit
#![cfg_attr(feature="clippy", allow(redundant_closure_call))]// not redundant in macros
#![cfg_attr(feature="clippy", allow(cast_lossless))]// too much noise (and too verbose)
// precedence: I prefer spaces to parentheses, but it's nice to recheck.

mod errors;
mod traits;
mod utf8_char;
mod utf8_iterator;
mod utf16_char;
mod utf16_iterator;

pub use traits::CharExt;
pub use utf8_char::Utf8Char;
pub use utf16_char::Utf16Char;
pub use utf8_iterator::Utf8Iterator;
pub use utf16_iterator::Utf16Iterator;
pub use traits::U8UtfExt;
pub use traits::U16UtfExt;

pub mod error {// keeping the public interface in one file
    //! Errors returned by various conversion methods in this crate.
    pub use utf8_char::{FromStrError, EmptyStrError};
    pub use errors::{InvalidCodepoint, InvalidUtf8};
    pub use errors::{InvalidUtf8FirstByte,InvalidUtf16FirstUnit};
    pub use errors::{InvalidUtf8Slice,InvalidUtf16Slice};
    pub use errors::{InvalidUtf8Array,InvalidUtf16Tuple};
}
