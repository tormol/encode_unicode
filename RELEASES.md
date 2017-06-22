Version 0.3.1 (2017-06-16)
==========================
* Implement `Display` for `Utf8Char` and `Utf16Char`.

Version 0.3.0 (2017-03-29)
==========================
* Replace the "no_std" feature with opt-out "std".
  * Upgrade ascii to v0.8.
  * Make tests compile on stable.
* Remove `CharExt::write_utf{8,16}()` because `encode_utf{8,16}()` has been stabilized.
* Return a proper error from `U16UtfExt::utf16_needs_extra_unit()` instead of `None`.
* Rename `U16UtfExt::utf_is_leading_surrogate()` to `is_utf16_leading_surrogate()`.
* Rename `Utf16Char::from_slice()` to `from_slice_start()`  and `CharExt::from_utf{8,16}_slice()`
  to `from_utf{8,16}_slice_start()` to be consistent with `Utf8Char`.
* Fix a bug where `CharExt::from_slice()` would accept some trailing surrogates
  as standalone codepoints.

Version 0.2.0 (2016-07-24)
==========================
* Change `CharExt::write_utf{8,16}()` to panic instead of returning `None`
  if the slice is too short.
* Fix bug where `CharExt::write_utf8()` and `Utf8Char::to_slice()` could change bytes it shouldn't.
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
