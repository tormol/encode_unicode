Version 0.2.0 (2016-07-24)
==========================
* Change `CharExt::write_utf{8,16}()` to panic instead of returning `None`
  if the slice is too short.
* Fix bug where `CharExt::write_utf8()` and `Utf8Char::to_slice()` could change bytes it should'nt.
* Rename lots of errors with search and replace:
  * CodePoint -> Codepoint
  * Several -> Multiple
* Update the ascii feature to use [ascii](https://tomprogrammer.github.io/rust-ascii/ascii/index.html) v0.7.
* Support `#[no_std]`; see 70e090ee for differences.
* Ungate impls of `AsciiExt`. (doesn't require ascii or nightly)
* Make the tests compile (and pass) again.
  (They still require nightly).

Version 0.1.* (2016-04-07)
==========================
First release.
