#![no_main]
#[macro_use]
extern crate libfuzzer_sys;
extern crate encode_unicode;

use encode_unicode::{IterExt, SliceExt, U8UtfExt, Utf8Char};
use encode_unicode::error::{InvalidUtf8Slice::*, InvalidUtf8::*};
use std::str;

fuzz_target!(|data: &[u8]| {
    let from_bytes: Vec<_> = data.iter().to_utf8chars().collect();

    let mut byte_start = 0;
    let mut item_start = 0;
    loop {
        let (valid_up_to, error_length) = match str::from_utf8(&data[byte_start..]) {
            Ok(s) => (s.len(), None),
            Err(e) => (e.valid_up_to(), e.error_len()),
        };
        let valid_range = byte_start..byte_start+valid_up_to;
        let good_part = str::from_utf8(&data[valid_range]).unwrap();
        let mut chars = 0;
        for (i,c) in good_part.chars().enumerate() {
            chars += 1;
            assert_eq!(from_bytes.get(item_start+i), Some(&Ok(Utf8Char::from(c))));
        }

        let error_start = item_start + chars;
        if let Some(error_length) = error_length {
            let error_end = error_start + error_length;
            assert!(from_bytes[error_start..error_end].iter().all(|r| r.is_err() ));
            item_start = error_end;
            byte_start = byte_start + valid_up_to + error_length;
        } else if byte_start + valid_up_to == data.len() {
            assert_eq!(from_bytes.len(), error_start);
            break;
        } else {
            let extra = data[byte_start + valid_up_to].extra_utf8_bytes().unwrap();
            assert_eq!(from_bytes.len() - error_start, data.len() - valid_up_to - byte_start);
            assert_eq!(from_bytes[error_start], Err(TooShort(1+extra)));
            break;
        }
    }

    let from_slice: Vec<_> = data.utf8char_indices().map(|(_,r,_)| r ).collect();
    for (i, (&br, &sr)) in from_bytes.iter().zip(&from_slice).enumerate() {
        match sr {
            // the slice-based iterator might detect too short earlier,
            // but that should be the only difference
            Err(TooShort(_)) | Err(Utf8(NotAContinuationByte(_)))
                => assert!(br.is_err(), "byte {}", i),
            _ => assert_eq!(sr, br, "byte {}", i),
        }
    }
    assert_eq!(from_slice.len(), from_bytes.len());
});
