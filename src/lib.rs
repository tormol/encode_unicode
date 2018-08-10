/* Copyright 2016 The encode_unicode Developers
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

#![cfg_attr(not(feature="std"), no_std)]

#![warn(missing_docs)]
#![cfg_attr(feature="cargo-clippy", allow(
    inconsistent_digit_grouping,
    large_digit_groups,// I sometimes group into UTF-8 control part and codepoint part
    derive_hash_xor_eq,// tested
    len_without_is_empty,// tha character types are never empty
    needless_return,// `foo.bar();\n foo` looks unfinished
    redundant_closure,// looks weird just passing the name of an enum variant
    redundant_closure_call,// not redundant in macros
    cast_lossless,// the sizes are part of the struct name and so won't change
    many_single_char_names,// the variables are in different scopes
    needless_range_loop,// the suggested iterator chains are less intuitive
    trivially_copy_pass_by_ref,// compatibility with char methods originally from AsciiExt
    identity_op,// applying a set of opereations with varying arguments to many elements looks nice
))]
#![cfg_attr(feature="cargo-clippy", warn(doc_markdown, filter_map))]
// opt-in lints that might be interesting to recheck once in a while:
//#![cfg_attr(feature="cargo-clippy", warn(result_unwrap_used, option_unwrap_used))]

mod errors;
mod traits;
mod utf8_char;
mod utf8_iterators;
mod utf16_char;
mod utf16_iterators;
mod decoding_iterators;

pub use traits::{CharExt, U8UtfExt, U16UtfExt, StrExt, IterExt, SliceExt};
pub use utf8_char::Utf8Char;
pub use utf16_char::Utf16Char;
pub use utf8_iterators::{Utf8Iterator, iter_bytes};
pub use utf16_iterators::{Utf16Iterator, iter_units};

pub mod error {// keeping the public interface in one file
    //! Errors returned by various conversion methods in this crate.
    pub use errors::{FromStrError, EmptyStrError};
    pub use errors::{InvalidCodepoint, InvalidUtf8};
    pub use errors::{InvalidUtf8FirstByte,InvalidUtf16FirstUnit};
    pub use errors::{InvalidUtf8Slice,InvalidUtf16Slice};
    pub use errors::{InvalidUtf8Array,InvalidUtf16Tuple};
    pub use errors::Utf16PairError;
}

pub mod iterator {
    //! Iterator types that you should rarely need to name
    pub use utf8_iterators::{Utf8Iterator, Utf8CharSplitter, Utf8Chars, Utf8CharIndices};
    pub use utf16_iterators::{Utf16Iterator, Utf16CharSplitter, Utf16Chars, Utf16CharIndices};
    pub use decoding_iterators::{Utf8CharMerger, Utf8CharDecoder};
    pub use decoding_iterators::{Utf16CharMerger, Utf16CharDecoder};
}
