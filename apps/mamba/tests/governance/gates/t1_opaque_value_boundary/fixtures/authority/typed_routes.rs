use std::num::NonZeroU64;

fn make_packet(id: NonZeroU64) -> super::PackedWord {
    emit_packet(id)
}

fn inspect_packet(word: super::PackedWord) -> Result<super::Decoded, super::CodecError> {
    read_packet(word)
}
