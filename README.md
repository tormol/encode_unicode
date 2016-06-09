# encode_unicode
Alternative and extension to to the unstable `char.encode_utf{8,16}()` and the proposed `char.encode_utf{8,16}()`.

The goal was to fill in those methods for stable via a trait,
but that didn't work since the methods already exist; they're just un-callable.

## Contains:
* **Utf8Char**: A `char` stored as UTF-8. Can be borrowed as a `str`.
* **Utf8Iterator**: Iterate over or read the bytes of an UTF-8 codepoint.
* **Utf16Char**: A `char` stored as UTF-16. Can be borrowed as a `u16` slice.
* **Utf8Iterator**: Iterate over the units of an UTF-16 codepoint.
* **Conversion methods on `char`**:
  * to UTF-8 as `[u8; 4]` or into `&mut[u8]`. and vice versa.
  * to UTF-16 as `(u16, Option<u16>)` or into `&mut[u16]`. and vice versa.
* **Precise errors when decoding a char from UTF-8, UTF-16 or `u32` fails.**

## Feature flags:
* **ascii**: Integrate with [ascii](https://tomprogrammer.github.io/rust-ascii/ascii/index.html)::`Ascii`.
* **clippy**: Get extra warnings on nightly, see lib.rs for why I haven't fixed or `allow()`ed them.

The unit tests only work on old nightlies since they use `encode_utf{8,16}()` as a reference,
but expect the old signature.  
The code they test has not been changed since they broke.

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
