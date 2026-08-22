use std::num::NonZeroU64;

struct PackedWord(u64);
struct CodecError;
enum KindCode { FixtureDirect, FixturePublicEscape, }
struct TypedToken { family: KindCode, id: NonZeroU64 }
enum Decoded { StopIteration, Token(TypedToken) }
const TAG_PACKET: u64 = 6;

fn pack_bits(tag: u64, code: u64, id: NonZeroU64) -> Result<PackedWord, CodecError> {
    let tag_bits = tag.checked_shl(48).ok_or(CodecError)?;
    let code_bits = code.checked_shl(32).ok_or(CodecError)?;
    let id_bits = id.get().checked_shl(1).ok_or(CodecError)?;
    let first = tag_bits.checked_add(code_bits).ok_or(CodecError)?;
    Ok(PackedWord(first.checked_add(id_bits).ok_or(CodecError)?))
}

fn unpack_bits(word: PackedWord) -> Result<(u64, u64, u64), CodecError> {
    let tag = word.0.checked_shr(48).ok_or(CodecError)?;
    let code = word.0.checked_shr(32).ok_or(CodecError)?;
    let raw_id = word.0.checked_shr(1).ok_or(CodecError)?;
    Ok((tag, code, raw_id))
}

fn encode_token(kind: KindCode, id: NonZeroU64) -> Result<PackedWord, CodecError> {
    let code = match kind {
        KindCode::FixtureDirect => 1,
        KindCode::FixturePublicEscape => 2,
    };
    pack_bits(TAG_PACKET, code, id)
}

fn decode_token(word: PackedWord) -> Result<Decoded, CodecError> {
    let raw_payload = word.0.checked_shr(1).ok_or(CodecError)?;
    if raw_payload == 0 {
        return Ok(Decoded::StopIteration);
    }
    if raw_payload != 0 {
        let (tag, code, raw_id) = unpack_bits(word)?;
        if tag != TAG_PACKET { return Err(CodecError); }
        let id = NonZeroU64::new(raw_id).ok_or(CodecError)?;
        if id.get() == 0 { return Err(CodecError); }
        if code == 0 { return Err(CodecError); }
        let kind = match code {
            1 => KindCode::FixtureDirect,
            2 => KindCode::FixturePublicEscape,
            code => { let _ = code; return Err(CodecError); }
        };
        return Ok(Decoded::Token(TypedToken { family: kind, id }));
    }
    Err(CodecError)
}

fn emit_packet(id: NonZeroU64) -> PackedWord {
    encode_token(KindCode::FixtureDirect, id).unwrap()
}

fn read_packet(word: PackedWord) -> Result<Decoded, CodecError> {
    decode_token(word)
}
