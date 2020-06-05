use crate::voice::input::{
    cached::{
        CompressedSource,
        MemorySource,
    },
    Container,
    Input,
    InputTypeData,
    ReadAudioExt,
    Reader,
};
use std::io::{Cursor, Read};

#[test]
fn memory_source_output_matches_input() {
    let input_bytes: Vec<u8> = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 20, 25, 64];
    let input = Input::new(false, Reader::Extension(Box::new(Cursor::new(input_bytes.clone()))), InputTypeData::FloatPcm, Container::Raw, None);
    let mut src = MemorySource::new(input, None);

    let mut cursor = 0;
    let mut output_bytes = vec![0u8; input_bytes.len()];

    while cursor < output_bytes.len() {
        if let Ok(sz) = src.read(&mut output_bytes[cursor..]) {
            cursor += sz;
        }
    }

    assert_eq!(input_bytes, output_bytes);

    let mut test_buf = [0u8; 1];
    assert!(matches!(src.read(&mut test_buf[..]), Ok(0usize)));
    assert_eq!(&test_buf, &[0u8]);
}

#[test]
fn memory_source_wont_read_past_end() {
    let input_bytes: Vec<u8> = vec![0, 1, 2, 3, 4];
    let input = Input::new(false, Reader::Extension(Box::new(Cursor::new(input_bytes.clone()))), InputTypeData::FloatPcm, Container::Raw, None);
    let mut src = MemorySource::new(input, None);

    let _ = src.consume(input_bytes.len());

    let mut test_buf = [0u8; 1];
    assert!(matches!(src.read(&mut test_buf[..]), Ok(0usize)));
    assert_eq!(&test_buf, &[0u8]);
}
