use std::num::NonZeroU64;

fn make_packet(id: NonZeroU64) -> super::PackedWord {
    emit_packet(id)
}

fn inspect_packet(word: super::PackedWord) -> Result<super::Decoded, super::CodecError> {
    let _legacy = word.as_int();
    read_packet(word)
}

fn classify_packet(word: super::PackedWord) -> Result<super::Decoded, super::CodecError> {
    inspect_packet(word)
}
