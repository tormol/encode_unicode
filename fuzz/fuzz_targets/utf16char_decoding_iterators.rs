#![no_main]
#[macro_use] extern crate libfuzzer_sys;
extern crate encode_unicode;

use encode_unicode::{IterExt, SliceExt, Utf16Char};
use std::char;

fuzz_target!(|data: &[u8]| {
    if data.len() % 2 != 0 {
        return;
    }
    let data = (0..data.len()/2).into_iter()
        .map(|i| ((data[i*2] as u16) << 8)  |  (data[i*2+1] as u16) )
        .collect::<Vec<u16>>();

    let from_units: Vec<_> = data.iter().to_utf16chars().collect();
    let from_slice: Vec<_> = data.utf16char_indices().collect();

    let mut surrogates = 0;
    for (i, (&ur, &(offset,sr,len))) in from_units.iter().zip(&from_slice).enumerate() {
        assert_eq!(sr, ur, "{} (data: +{})", i, surrogates);
        assert_eq!(offset, i+surrogates);
        let unit = data[i+surrogates];
        if let Some(c) = char::from_u32(unit as u32) {
            assert_eq!(ur, Ok(Utf16Char::from(c)), "{} (data: +{})", i, surrogates);
            assert_eq!(len, 1);
        } else {
            assert_eq!(char::from_u32(unit as u32), None);
            if let Ok(u16c) = ur {
                surrogates += 1;
                assert_eq!(char::from_u32(data[i+surrogates] as u32), None);
                assert_eq!(len, 2);
                assert!(u16c.to_char() > '\u{ffff}');
            } else {
                assert_eq!(len, 1);
            }
        }
    }
    assert_eq!(from_units.len(), data.len()-surrogates);
    assert_eq!(from_slice.len(), data.len()-surrogates);
});
