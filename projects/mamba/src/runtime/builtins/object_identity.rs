use super::super::value::MbValue;

/// id(value) — return unique identity of an object.
pub fn mb_id(val: MbValue) -> MbValue {
    if let Some(ptr) = val.as_ptr() {
        // Truncate to fit 48-bit signed int range
        MbValue::from_int((ptr as u64 & 0x0000_7FFF_FFFF_FFFF) as i64)
    } else {
        // For primitives, use the raw bits truncated
        MbValue::from_int((val.to_bits() >> 17) as i64)
    }
}
