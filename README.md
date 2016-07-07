# encode_unicode
Alternatives and extensions to to the unstable `char.encode_utf{8,16}()` methods.

## Features:
* **[Utf8Char](http://tormol.github.io/rustdoc/encode_unicode/struct.Utf8Char.html)**: A `char` stored as UTF-8. Can be borrowed as a `str`.
* **[Utf16Char](http://tormol.github.io/rustdoc/encode_unicode/struct.Utf16Char.html)**: A `char` stored as UTF-16. Can be borrowed as a `u16` slice.
* **[Conversion methods on `char`](http://tormol.github.io/rustdoc/encode_unicode/trait.CharExt.html)**:
  * to UTF-8 as `[u8; 4]` or into `&mut[u8]`. and vice versa.
  * to UTF-16 as `(u16, Option<u16>)` or into `&mut[u16]`. and vice versa.
* [Precise errors when decoding a char fro9m UTF-8, UTF-16 or `u32` fails](http://tormol.github.io/rustdoc/encode_unicode/error/index.html).

[See the [documentation for the remaining](http://tormol.github.io/rustdoc/encode_unicode/index.html).

The goal was to fill in those methods for stable via a trait,
but that didn't work since the methods already exist; they're just un-callable.

## Optional Features:
* **no_std**: Use `#[no_std]`; There are some differences:
  * `AsciiExt` doesn't exist, but `is_ascii()` is made available as an inherent impl.
  * `Error` doesn't exist, but `description()` is made available as an inherent impl.
  * There is no `io`, so `Utf8Iterator` doesn't implement `Read`.
  * The iterators doesn't implement `Debug`.
* **ascii**: Convert `Utf8Char` and `Utf16Char` to and from [ascii](https://tomprogrammer.github.io/rust-ascii/ascii/index.html)::[`AsciiChar`](https://tomprogrammer.github.io/rust-ascii/ascii/enum.AsciiChar.html).
* **ascii_no_std**: You need to use this feature instead of both ascii and no_std.  
  This is because the ascii crate needs to know about `#[no_std]`, but the features are otherwize independent.

The tests require nightly because they use compare against `encode_utf{8,16}()`,

## License

Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the Apache-2.0
license, shall be dual licensed as above, without any additional terms or
conditions.
