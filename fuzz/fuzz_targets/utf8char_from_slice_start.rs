#![no_main]
#[macro_use]
extern crate libfuzzer_sys;
extern crate encode_unicode;

use encode_unicode::Utf8Char;

fuzz_target!(|data: &[u8]| {
    if data.len() > 0 {
        // validate the result of encode_unicode against the std library
        match Utf8Char::from_slice_start(data) {
            Err(_) => assert!(std::str::from_utf8(data).is_err()),
            Ok((c, len)) => assert_eq!(c.as_str(), std::str::from_utf8(&data[..len]).unwrap()),
        }
    }
});
