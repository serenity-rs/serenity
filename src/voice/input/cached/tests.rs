use crate::voice::input::{
	cached::{
		CompressedSource,
		MemorySource,
	},
	AudioType,
	Input,
	Reader,
};
use std::io::{Cursor, Read};

#[test]
fn memory_source_output_matches_input() {
	let input_bytes: Vec<u8> = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 20, 25, 64];
	let input = Input::new(false, Reader::Extension(Box::new(Cursor::new(input_bytes.clone()))), AudioType::FloatPcm, None);
	let mut src = MemorySource::new(input, None);

	let mut cursor = 0;
	let mut output_bytes = vec![0u8; input_bytes.len()];

	while cursor < output_bytes.len() {
		if let Ok(sz) = src.read(&mut output_bytes[cursor..]) {
			cursor += sz;
		}
	}

	assert_eq!(input_bytes, output_bytes);
}
