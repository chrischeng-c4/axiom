use std::num::NonZeroU64;

struct Parcel(u64);
struct FrameError;
enum Flavor { Alpha, Beta, }
struct Token { kind: Flavor, serial: NonZeroU64 }
enum Outcome { Halt, Wrapped(Token) }
const TAG_FRAME: u64 = 6;

fn combine(marker: u64, opcode: u64, serial: NonZeroU64) -> Result<Parcel, FrameError> {
    let tag_limit = 1u64.checked_shl(16).ok_or(FrameError)?;
    let code_limit = 1u64.checked_shl(16).ok_or(FrameError)?;
    let id_limit = 1u64.checked_shl(31).ok_or(FrameError)?;
    if marker >= tag_limit || opcode >= code_limit || serial.get() >= id_limit { return Err(FrameError); }
    let marker_bits = marker.checked_shl(48).ok_or(FrameError)?;
    let marker_code_bits = opcode.checked_shl(32).ok_or(FrameError)?;
    let serial_bits = serial.get().checked_shl(1).ok_or(FrameError)?;
    let first = marker_bits.checked_add(marker_code_bits).ok_or(FrameError)?;
    Ok(Parcel(first.checked_add(serial_bits).ok_or(FrameError)?))
}

fn split(frame_word: Parcel) -> Result<(u64, u64), FrameError> {
    let marker = frame_word.0.checked_shr(48).ok_or(FrameError)?;
    let marker_bits = marker.checked_shl(48).ok_or(FrameError)?;
    let residue = frame_word.0.checked_sub(marker_bits).ok_or(FrameError)?;
    Ok((marker, residue))
}

fn seal(flavor: Flavor, serial: NonZeroU64) -> Result<Parcel, FrameError> {
    let opcode = match flavor {
        Flavor::Alpha => 1,
        Flavor::Beta => 2,
    };
    combine(TAG_FRAME, opcode, serial)
}

fn open(frame_word: Parcel) -> Result<Outcome, FrameError> {
    let (marker, residue) = split(frame_word)?;
    if marker != TAG_FRAME { return Err(FrameError); }
    if residue == 0 { return Ok(Outcome::Halt); }
    if residue != 0 {
        if residue & 1 != 0 { return Err(FrameError); }
        let opcode = residue.checked_shr(32).ok_or(FrameError)?;
        let marker_code_bits = opcode.checked_shl(32).ok_or(FrameError)?;
        let residue = residue.checked_sub(marker_code_bits).ok_or(FrameError)?;
        let raw_serial = residue.checked_shr(1).ok_or(FrameError)?;
        let serial = NonZeroU64::new(raw_serial).ok_or(FrameError)?;
        if serial.get() == 0 { return Err(FrameError); }
        if opcode == 0 { return Err(FrameError); }
        let flavor = match opcode {
            1 => Flavor::Alpha,
            2 => Flavor::Beta,
            opcode => { let _ = opcode; return Err(FrameError); }
        };
        return Ok(Outcome::Wrapped(Token { kind: flavor, serial: serial }));
    }
    Err(FrameError)
}

fn originate(serial: NonZeroU64) -> Parcel {
    seal(Flavor::Alpha, serial).unwrap()
}

fn acquire(frame_word: Parcel) -> Result<Outcome, FrameError> {
    open(frame_word)
}
