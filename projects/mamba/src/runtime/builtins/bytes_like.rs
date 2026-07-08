use super::super::rc::ObjData;
use super::super::value::MbValue;

// HANDWRITE-BEGIN reason: Phase 1.5 cross-cutting fix (#11) — Python's
// bytes/bytearray/memoryview/array('B') interconvert under `==`. Mamba's
// runtime stores these four representations differently (Bytes, ByteArray,
// Instance(memoryview, _buffer=...), Dict{__class__: "array", data: List[int]}),
// so structural equality must unify them before falling through to
// dispatch. Codegen has no section type for buffer-protocol coercion yet
// — convert to CODEGEN once the standardize sweep grows one.

/// Coerce a Python bytes-like MbValue into a byte vector.
///
/// Returns `Some` for:
///   * `bytes`              -> direct copy
///   * `bytearray`          -> read-lock copy of the contained Vec<u8>
///   * `memoryview(bb)`     -> recurse into the Instance's `_buffer` field
///   * `array('B'|'b', xs)` -> Dict-flavoured array whose `data` list holds
///                             ints; truncated to u8 bytes.
///
/// Returns `None` for anything else (caller can fall through to regular
/// equality / dispatch). Side-effect-free, safe to call from comparison
/// paths.
pub fn try_bytes_like(v: MbValue) -> Option<Vec<u8>> {
    if let Some(id) = v.as_int() {
        if crate::runtime::stdlib::array_mod::is_array_handle(id as u64) {
            return try_bytes_like(crate::runtime::stdlib::array_mod::mb_array_tobytes(v));
        }
    }
    let ptr = v.as_ptr()?;
    unsafe {
        match &(*ptr).data {
            ObjData::Bytes(b) => Some(b.clone()),
            ObjData::ByteArray(lock) => Some(lock.read().unwrap().clone()),
            ObjData::Instance { class_name, fields } => {
                if class_name != "memoryview" {
                    return None;
                }
                let buf = fields.read().unwrap().get("_buffer").copied()?;
                try_bytes_like(buf)
            }
            ObjData::Dict(lock) => {
                let map = lock.read().unwrap();
                let class_v = map.get("__class__").copied()?;
                let cp = class_v.as_ptr()?;
                let is_array = matches!(&(*cp).data, ObjData::Str(s) if s == "array");
                if !is_array {
                    return None;
                }
                let typecode_v = map.get("typecode").copied();
                let typecode = typecode_v
                    .and_then(|tv| tv.as_ptr())
                    .and_then(|tp| match &(*tp).data {
                        ObjData::Str(s) => Some(s.clone()),
                        _ => None,
                    })
                    .unwrap_or_default();
                if typecode != "B" && typecode != "b" {
                    return None;
                }
                let data_v = map.get("data").copied()?;
                let dp = data_v.as_ptr()?;
                let ObjData::List(items_lock) = &(*dp).data else {
                    return None;
                };
                let items = items_lock.read().unwrap();
                let mut out = Vec::with_capacity(items.len());
                for item in items.iter() {
                    let i = item.as_int()?;
                    out.push(i as u8);
                }
                Some(out)
            }
            _ => None,
        }
    }
}
// HANDWRITE-END
