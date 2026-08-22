use std::num::NonZeroU64;
fn make_packet(kind: super::KindCode, id: NonZeroU64) -> super::PackedWord { emit_packet(kind, id) }
fn inspect_packet(word: super::PackedWord) -> Result<super::Decoded, super::CodecError> { read_packet(word) }
fn classify_packet(word: super::PackedWord) -> Result<super::Decoded, super::CodecError> { inspect_packet(word) }
