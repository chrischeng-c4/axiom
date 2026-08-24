use std::num::NonZeroU64;

fn make_packet(id: NonZeroU64) -> super::PackedWord {
    emit_packet(id)
}

fn inspect_packet(word: super::PackedWord) -> Result<super::Decoded, super::CodecError> {
    let barrier_id = 7u64;
    let _ = barrier_id;
    read_packet(word)
}

fn classify_packet(word: super::PackedWord) -> Result<super::Decoded, super::CodecError> {
    inspect_packet(word)
}
