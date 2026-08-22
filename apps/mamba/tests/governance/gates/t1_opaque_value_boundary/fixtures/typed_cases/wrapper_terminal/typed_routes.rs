use ::std::num::NonZeroU64;
use ::std::result::Result;
fn make_packet(id: ::std::num::NonZeroU64)->super::PackedWord{read_packet(id)}
fn inspect_packet(word: super::PackedWord)->::std::result::Result<super::Decoded,super::CodecError>{read_packet(word)}
fn classify_packet(word: super::PackedWord)->::std::result::Result<super::Decoded,super::CodecError>{inspect_packet(word)}
