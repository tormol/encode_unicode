/* Copyright 2016 Torbjørn Birch Moltu
 *
 * Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
 * http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
 * http://opensource.org/licenses/MIT>, at your option. This file may not be
 * copied, modified, or distributed except according to those terms.
 */


/*!
Alternatives and extensions to the unstable `char.encode_utf8()` and `char.encode_utf16()`.

[crates.io page](https://crates.io/crates/encode_unicode)
[github repository](https://github.com/tormol/encode_unicode)

# Optional features:
* **no_std**: Use `#[no_std]`; There are some differences:
  * `AsciiExt` doesn't exist, but `is_ascii()` is made available as an inherent impl.
  * `Error` doesn't exist, but `description()` is made available as an inherent impl.
  * There is no `io`, so `Utf8Iterator` doesn't implement `Read`.
  * The iterators doesn't implement `Debug`.
* **ascii**: Convert `Utf8Char` and `Utf16Char` to and from [ascii](https://tomprogrammer.github.io/rust-ascii/ascii/index.html)::[`AsciiChar`](https://tomprogrammer.github.io/rust-ascii/ascii/enum.AsciiChar.html).
* **ascii_no_std**: You need to use this feature instead of both ascii and no_std.
  This is because the ascii crate needs to know about `#[no_std]`, but the features are otherwize independent.
*/


#![cfg_attr(feature="std", warn(missing_docs))] // Don't bother documenting std standins.

#![cfg_attr(not(feature="std"), no_std)]
// either `cargo clippy` doesn't see theese, or I get a warning when I build.
#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]
#![cfg_attr(feature="clippy", allow(len_without_is_empty))]// UtfxChar is never empty
#![cfg_attr(feature="clippy", allow(match_same_arms))]
#![cfg_attr(feature="clippy", allow(derive_hash_xor_eq))]// tested
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
    pub use utf8_char::FromStrError;
    pub use errors::{InvalidCodepoint};
    pub use errors::{InvalidUtf8FirstByte,InvalidUtf8};
    pub use errors::{InvalidUtf8Slice,InvalidUtf16Slice};
    pub use errors::{InvalidUtf8Array,InvalidUtf16Tuple};
}
