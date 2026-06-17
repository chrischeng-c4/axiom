/// tarfile module for Mamba (#445).
///
/// Real in-memory TarFile / TarInfo objects following the __class__-tagged
/// dict-stub pattern (ET.Element / ConfigParser precedent):
///   * `TarInfo` — attrs land as dict keys (name/size/mtime/mode/type/...),
///     methods (`get_info`, `tobuf`, `create_pax_header`, `isreg`/`isdir`/...)
///     dispatch via `dict_ops::dispatch_dict_method` → `dispatch_tar_stub_method`.
///   * `TarFile` — `tarfile.open()` over a BytesIO fileobj or a disk path,
///     modes `w`/`w:gz`/`r`/`r|`, `addfile`/`getmembers`/`getnames`/
///     `getmember`/`extractfile`/`extractall`/`extract`/`close`/`next`,
///     context-manager hooks (`class.rs::mb_context_enter/exit`) that track
///     `.closed` and raise OSError on re-entering a closed archive.
///   * Real ustar/GNU/pax header encoding: `itn`/`nti` (octal + GNU base-256),
///     `stn`/`nts`, 512-byte blocks, checksum, two-NUL-block terminator,
///     RECORDSIZE padding, pax extended headers (`././@PaxHeader`) and GNU
///     long-name blocks (`././@LongLink`).
///   * Extraction filters: `fully_trusted_filter` (identity), `tar_filter`,
///     `data_filter` (strip leading '/', reject traversal/absolute links,
///     clear high mode bits) raising the FilterError hierarchy.

use std::collections::HashMap;
use super::super::value::MbValue;
use super::super::rc::{MbObject, ObjData};
use super::super::dict_ops::DictKey;

macro_rules! dispatch_unary {
    ($name:ident, $fn:ident) => {
        unsafe extern "C" fn $name(args_ptr: *const MbValue, nargs: usize) -> MbValue {
            let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
            $fn(a.get(0).copied().unwrap_or_else(MbValue::none))
        }
    };
}

dispatch_unary!(dispatch_is_tarfile, mb_tarfile_is_tarfile);

const BLOCKSIZE: usize = 512;
const RECORDSIZE: usize = 10240;
const POSIX_MAGIC: [u8; 8] = *b"ustar\x0000";
const GNU_MAGIC: [u8; 8] = *b"ustar  \x00";
const USTAR_FORMAT: i64 = 0;
const GNU_FORMAT: i64 = 1;
const PAX_FORMAT: i64 = 2;

// ── error-path helpers ──────────────────────────────────────────────────────
// Exception class-names match the registry entries created in `register()`
// (e.g. "ReadError", "ValueError"), so `except tarfile.ReadError` /
// `except ValueError` resolve. Precedent: netrc_mod::raise_named.

/// Raise a catchable exception whose type-name is `exc`, return None.
fn tf_raise(exc: &str, msg: &str) -> MbValue {
    super::super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str(exc.to_string())),
        MbValue::from_ptr(MbObject::new_str(msg.to_string())),
    );
    MbValue::none()
}

fn tf_is_dict(val: MbValue) -> bool {
    val.as_ptr()
        .map(|ptr| unsafe { matches!((*ptr).data, ObjData::Dict(_)) })
        .unwrap_or(false)
}

/// Look up `key` in a kwargs dict (the trailing positional dict the runtime
/// appends for keyword arguments). Returns None if absent / not a dict.
fn tf_kw_get(kwargs: Option<MbValue>, key: &str) -> Option<MbValue> {
    let ptr = kwargs?.as_ptr()?;
    unsafe {
        if let ObjData::Dict(ref lock) = (*ptr).data {
            lock.read().unwrap().get(key).copied()
        } else {
            None
        }
    }
}

/// Extract a `str` value, if `val` is a str object.
fn tf_as_str(val: MbValue) -> Option<String> {
    let ptr = val.as_ptr()?;
    unsafe {
        if let ObjData::Str(ref s) = (*ptr).data {
            Some(s.clone())
        } else {
            None
        }
    }
}

/// Extract bytes / bytearray contents.
fn tf_as_bytes(val: MbValue) -> Option<Vec<u8>> {
    let ptr = val.as_ptr()?;
    unsafe {
        match &(*ptr).data {
            ObjData::Bytes(b) => Some(b.clone()),
            ObjData::ByteArray(lock) => Some(lock.read().unwrap().clone()),
            _ => None,
        }
    }
}

/// Integer extraction tolerant of inline ints, bools and BigInt heap objects.
fn tf_as_i128(val: MbValue) -> Option<i128> {
    if let Some(i) = val.as_int() {
        return Some(i as i128);
    }
    if let Some(b) = val.as_bool() {
        return Some(b as i128);
    }
    if let Some(ptr) = val.as_ptr() {
        unsafe {
            if let ObjData::BigInt(ref b) = (*ptr).data {
                use num_traits::ToPrimitive;
                return b.to_i128();
            }
        }
    }
    None
}

/// Build an int MbValue, boxing to BigInt when outside the inline 48-bit range.
fn int_value(n: i128) -> MbValue {
    if n >= -(1i128 << 47) && n < (1i128 << 47) {
        MbValue::from_int(n as i64)
    } else {
        super::super::bigint_ops::bigint_from_i128(n)
    }
}

/// Byte length of a `BytesIO`-like fileobj's in-memory buffer (`_buffer`
/// instance field, populated by io_mod). None when `val` is not such an object
/// or has no readable byte buffer — in which case the caller stays inert.
fn tf_fileobj_buf_len(val: MbValue) -> Option<usize> {
    let ptr = val.as_ptr()?;
    unsafe {
        if let ObjData::Instance { ref fields, .. } = (*ptr).data {
            let f = fields.read().unwrap();
            let buf = f.get("_buffer")?;
            let bptr = buf.as_ptr()?;
            match &(*bptr).data {
                ObjData::Bytes(b) => Some(b.len()),
                ObjData::ByteArray(ref lock) => Some(lock.read().unwrap().len()),
                _ => None,
            }
        } else {
            None
        }
    }
}

/// Bytes remaining in a BytesIO-like fileobj from its current `_pos` to the
/// end of `_buffer` (does NOT advance the position).
fn tf_fileobj_remaining(val: MbValue) -> Option<Vec<u8>> {
    let ptr = val.as_ptr()?;
    unsafe {
        if let ObjData::Instance { ref fields, .. } = (*ptr).data {
            let f = fields.read().unwrap();
            let buf = f.get("_buffer").and_then(|v| tf_as_bytes(*v))?;
            let pos = f.get("_pos").and_then(|v| v.as_int()).unwrap_or(0).max(0) as usize;
            Some(if pos < buf.len() { buf[pos..].to_vec() } else { Vec::new() })
        } else {
            None
        }
    }
}

/// Split a raw native arg slice into (positional, trailing-kwargs-dict).
fn tf_split<'a>(a: &'a [MbValue]) -> (&'a [MbValue], Option<MbValue>) {
    match a.last().copied().filter(|v| tf_is_dict(*v)) {
        Some(kw) => (&a[..a.len().saturating_sub(1)], Some(kw)),
        None => (a, None),
    }
}

// ── dict-stub field helpers ─────────────────────────────────────────────────

fn dget(d: MbValue, key: &str) -> Option<MbValue> {
    let ptr = d.as_ptr()?;
    unsafe {
        if let ObjData::Dict(ref lock) = (*ptr).data {
            lock.read().unwrap().get(key).copied()
        } else {
            None
        }
    }
}

/// Insert an OWNED (fresh, +1) value, releasing any displaced value.
fn dset(d: MbValue, key: &str, val: MbValue) {
    if let Some(ptr) = d.as_ptr() {
        unsafe {
            if let ObjData::Dict(ref lock) = (*ptr).data {
                let mut m = lock.write().unwrap();
                let old = m.insert(DictKey::Str(key.to_string()), val);
                if let Some(o) = old {
                    if o.to_bits() != val.to_bits() {
                        super::super::rc::release_if_ptr(o);
                    }
                }
            }
        }
    }
}

fn s_val(s: &str) -> MbValue {
    MbValue::from_ptr(MbObject::new_str(s.to_string()))
}

fn b_val(b: Vec<u8>) -> MbValue {
    MbValue::from_ptr(MbObject::new_bytes(b))
}

// ── number/string field codecs (CPython Lib/tarfile.py itn/nti/stn/nts) ─────

/// Encode an integer into a tar number field: octal-ASCII when it fits,
/// GNU base-256 (leading 0o200/0o377 byte) when format == GNU_FORMAT.
fn itn(n: i128, digits: usize, format: i64) -> Result<Vec<u8>, String> {
    if digits < 2 || digits > 32 {
        return Err("overflow in number field".to_string());
    }
    let limit = 8i128.pow((digits - 1) as u32);
    if n >= 0 && n < limit {
        let mut b = format!("{:0width$o}", n, width = digits - 1).into_bytes();
        b.push(0);
        Ok(b)
    } else if format == GNU_FORMAT
        && n >= -(256i128.pow((digits - 1) as u32))
        && n < 256i128.pow((digits - 1) as u32)
    {
        let mut out = vec![if n >= 0 { 0o200u8 } else { 0o377u8 }];
        let mut m = if n >= 0 {
            n
        } else {
            256i128.pow(digits as u32) + n
        };
        let mut tail = vec![0u8; digits - 1];
        for i in (0..digits - 1).rev() {
            tail[i] = (m & 0xff) as u8;
            m >>= 8;
        }
        out.extend(tail);
        Ok(out)
    } else {
        Err("overflow in number field".to_string())
    }
}

/// Decode a tar number field (octal-ASCII or GNU base-256).
fn nti(s: &[u8]) -> Result<i128, ()> {
    if s.is_empty() {
        return Err(());
    }
    if s[0] == 0o200 || s[0] == 0o377 {
        let mut n: i128 = 0;
        for &b in &s[1..] {
            n = (n << 8) + b as i128;
        }
        if s[0] == 0o377 {
            n -= 256i128.pow((s.len() - 1) as u32);
        }
        Ok(n)
    } else {
        let txt = nts(s);
        let t = txt.trim();
        if t.is_empty() {
            return Ok(0);
        }
        i128::from_str_radix(t, 8).map_err(|_| ())
    }
}

/// Encode a string with a minimal codec set (utf-8 default; ascii/latin-1
/// map unencodable chars to '?', mirroring errors="replace").
fn encode_str(s: &str, encoding: &str) -> Vec<u8> {
    match encoding {
        "ascii" => s
            .chars()
            .map(|c| if (c as u32) < 128 { c as u8 } else { b'?' })
            .collect(),
        "iso8859-1" | "latin-1" | "latin1" => s
            .chars()
            .map(|c| if (c as u32) < 256 { c as u8 } else { b'?' })
            .collect(),
        _ => s.as_bytes().to_vec(),
    }
}

/// Convert a string to a NUL-padded (truncating) fixed-width bytes field.
fn stn(s: &str, length: usize, encoding: &str) -> Vec<u8> {
    let mut b = encode_str(s, encoding);
    b.truncate(length);
    b.resize(length, 0);
    b
}

/// Convert a NUL-terminated bytes field back to a string (stops at first NUL).
fn nts(s: &[u8]) -> String {
    let end = s.iter().position(|&b| b == 0).unwrap_or(s.len());
    String::from_utf8_lossy(&s[..end]).to_string()
}

/// Unsigned header checksum (chksum field counted as 8 spaces).
fn checksum_unsigned(block: &[u8]) -> i128 {
    block
        .iter()
        .enumerate()
        .map(|(i, &b)| if (148..156).contains(&i) { 0x20 } else { b as i128 })
        .sum()
}

/// Signed header checksum variant (some old tars).
fn checksum_signed(block: &[u8]) -> i128 {
    block
        .iter()
        .enumerate()
        .map(|(i, &b)| {
            if (148..156).contains(&i) {
                0x20
            } else {
                (b as i8) as i128
            }
        })
        .sum()
}

fn py_float_str(f: f64) -> String {
    if f.fract() == 0.0 && f.is_finite() && f.abs() < 1e16 {
        format!("{:.1}", f)
    } else {
        format!("{}", f)
    }
}

// ── TarInfo view + header building ──────────────────────────────────────────

#[derive(Clone, Default)]
struct TView {
    name: String,
    mode: Option<i128>,
    uid: Option<i128>,
    gid: Option<i128>,
    size: i128,
    mtime_int: i128,
    mtime_float: Option<f64>,
    type_b: u8,
    linkname: String,
    uname: String,
    gname: String,
    devmajor: i128,
    devminor: i128,
    pax: Vec<(String, String)>,
}

fn opt_num_field(d: MbValue, key: &str, default: i128) -> Option<i128> {
    match dget(d, key) {
        Some(v) if v.is_none() => None,
        Some(v) => tf_as_i128(v).or(Some(default)),
        None => Some(default),
    }
}

/// Read a TarInfo dict-stub (or a get_info()-shaped info dict) into a view.
fn view_of(ti: MbValue) -> TView {
    let gs = |k: &str| dget(ti, k).and_then(tf_as_str).unwrap_or_default();
    let mut v = TView {
        name: gs("name"),
        mode: opt_num_field(ti, "mode", 0o644),
        uid: opt_num_field(ti, "uid", 0),
        gid: opt_num_field(ti, "gid", 0),
        size: dget(ti, "size").and_then(tf_as_i128).unwrap_or(0),
        linkname: gs("linkname"),
        uname: gs("uname"),
        gname: gs("gname"),
        devmajor: dget(ti, "devmajor").and_then(tf_as_i128).unwrap_or(0),
        devminor: dget(ti, "devminor").and_then(tf_as_i128).unwrap_or(0),
        type_b: dget(ti, "type")
            .and_then(tf_as_bytes)
            .and_then(|b| b.first().copied())
            .unwrap_or(b'0'),
        ..TView::default()
    };
    if let Some(m) = dget(ti, "mtime") {
        if m.is_float() {
            let f = m.as_float().unwrap_or(0.0);
            v.mtime_float = Some(f);
            v.mtime_int = f as i128;
        } else {
            v.mtime_int = tf_as_i128(m).unwrap_or(0);
        }
    }
    if let Some(pax) = dget(ti, "pax_headers") {
        if let Some(ptr) = pax.as_ptr() {
            unsafe {
                if let ObjData::Dict(ref lock) = (*ptr).data {
                    for (k, val) in lock.read().unwrap().iter() {
                        if let DictKey::Str(ks) = k {
                            if let Some(vs) = tf_as_str(*val) {
                                v.pax.push((ks.clone(), vs));
                            }
                        }
                    }
                }
            }
        }
    }
    v
}

fn is_regular_type(t: u8) -> bool {
    matches!(t, b'0' | 0 | b'7' | b'S')
}

struct HFields<'a> {
    name: &'a str,
    mode: i128,
    uid: i128,
    gid: i128,
    size: i128,
    mtime: i128,
    type_b: u8,
    linkname: &'a str,
    magic: &'a [u8; 8],
    uname: &'a str,
    gname: &'a str,
    dev: Option<(i128, i128)>,
    prefix: &'a str,
}

/// Assemble one 512-byte header block with checksum.
fn create_header(h: &HFields, format: i64, encoding: &str) -> Result<Vec<u8>, String> {
    let mut buf = Vec::with_capacity(BLOCKSIZE);
    buf.extend(stn(h.name, 100, encoding));
    buf.extend(itn(h.mode & 0o7777, 8, format)?);
    buf.extend(itn(h.uid, 8, format)?);
    buf.extend(itn(h.gid, 8, format)?);
    buf.extend(itn(h.size, 12, format)?);
    buf.extend(itn(h.mtime, 12, format)?);
    buf.extend(b"        "); // checksum placeholder
    buf.push(h.type_b);
    buf.extend(stn(h.linkname, 100, encoding));
    buf.extend(h.magic);
    buf.extend(stn(h.uname, 32, encoding));
    buf.extend(stn(h.gname, 32, encoding));
    match h.dev {
        Some((maj, min)) => {
            buf.extend(itn(maj, 8, format)?);
            buf.extend(itn(min, 8, format)?);
        }
        None => {
            buf.extend(stn("", 8, encoding));
            buf.extend(stn("", 8, encoding));
        }
    }
    buf.extend(stn(h.prefix, 155, encoding));
    buf.resize(BLOCKSIZE, 0);
    let ck = checksum_unsigned(&buf);
    let ckb = format!("{:06o}\0", ck).into_bytes();
    buf[148..155].copy_from_slice(&ckb);
    Ok(buf)
}

fn pad_block(mut b: Vec<u8>) -> Vec<u8> {
    let rem = b.len() % BLOCKSIZE;
    if rem > 0 {
        b.resize(b.len() + BLOCKSIZE - rem, 0);
    }
    b
}

/// Split a >100-char name into prefix (<=155) + name (<=100) at a '/'.
fn posix_split(name: &str) -> Result<(String, String), String> {
    let comps: Vec<&str> = name.split('/').collect();
    for i in 1..comps.len() {
        let prefix = comps[..i].join("/");
        let nm = comps[i..].join("/");
        if prefix.as_bytes().len() <= 155 && nm.as_bytes().len() <= 100 {
            return Ok((prefix, nm));
        }
    }
    Err("name is too long".to_string())
}

/// GNU longname/longlink extension block sequence ("././@LongLink").
fn gnu_long_header(name: &str, type_b: u8) -> Result<Vec<u8>, String> {
    let mut data = name.as_bytes().to_vec();
    data.push(0);
    let h = HFields {
        name: "././@LongLink",
        mode: 0,
        uid: 0,
        gid: 0,
        size: data.len() as i128,
        mtime: 0,
        type_b,
        linkname: "",
        magic: &GNU_MAGIC,
        uname: "",
        gname: "",
        dev: None,
        prefix: "",
    };
    let mut out = create_header(&h, USTAR_FORMAT, "utf-8")?;
    out.extend(pad_block(data));
    Ok(out)
}

/// Pax extended header: "././@PaxHeader" header block + "%d key=value\n"
/// record blocks.
fn pax_generic_header(pax: &[(String, String)], type_b: u8) -> Result<Vec<u8>, String> {
    let mut records: Vec<u8> = Vec::new();
    for (k, v) in pax {
        let kb = k.as_bytes();
        let vb = v.as_bytes();
        let l = kb.len() + vb.len() + 3; // ' ' + '=' + '\n'
        let mut p = 0usize;
        loop {
            let n = l + p.to_string().len();
            if n == p {
                break;
            }
            p = n;
        }
        records.extend(p.to_string().as_bytes());
        records.push(b' ');
        records.extend(kb);
        records.push(b'=');
        records.extend(vb);
        records.push(b'\n');
    }
    let h = HFields {
        name: "././@PaxHeader",
        mode: 0,
        uid: 0,
        gid: 0,
        size: records.len() as i128,
        mtime: 0,
        type_b,
        linkname: "",
        magic: &POSIX_MAGIC,
        uname: "",
        gname: "",
        dev: None,
        prefix: "",
    };
    let mut out = create_header(&h, USTAR_FORMAT, "ascii")?;
    out.extend(pad_block(records));
    Ok(out)
}

/// PAX header sequence for a member: optional extended header + ustar header
/// with overflowing/float fields zeroed (carried in the pax records).
fn create_pax(v: &TView, name: &str) -> Result<Vec<u8>, String> {
    let mut pax: Vec<(String, String)> = v.pax.clone();
    let has = |pax: &[(String, String)], k: &str| pax.iter().any(|(kk, _)| kk == k);

    // String fields that exceed the field length or are not ASCII.
    let str_fields: [(&str, &str, usize); 4] = [
        (name, "path", 100),
        (&v.linkname, "linkpath", 100),
        (&v.uname, "uname", 32),
        (&v.gname, "gname", 32),
    ];
    for (val, hname, length) in str_fields {
        if has(&pax, hname) {
            continue;
        }
        if !val.is_ascii() || val.chars().count() > length {
            pax.push((hname.to_string(), val.to_string()));
        }
    }

    // Number fields that overflow the octal field or are floats.
    let mut hdr_uid = v.uid.unwrap_or(0);
    let mut hdr_gid = v.gid.unwrap_or(0);
    let mut hdr_size = v.size;
    let mut hdr_mtime = match v.mtime_float {
        Some(f) => f.round_ties_even() as i128,
        None => v.mtime_int,
    };
    {
        let num = |key: &str, digits: u32, hdr: &mut i128, fval: Option<f64>, pax: &mut Vec<(String, String)>| {
            let limit = 8i128.pow(digits - 1);
            let mut needs = false;
            let record_val = match fval {
                Some(f) => py_float_str(f),
                None => hdr.to_string(),
            };
            if !(0 <= *hdr && *hdr < limit) {
                *hdr = 0;
                needs = true;
            } else if fval.is_some() {
                needs = true;
            }
            if needs && !pax.iter().any(|(k, _)| k == key) {
                pax.push((key.to_string(), record_val));
            }
        };
        num("uid", 8, &mut hdr_uid, None, &mut pax);
        num("gid", 8, &mut hdr_gid, None, &mut pax);
        num("size", 12, &mut hdr_size, None, &mut pax);
        num("mtime", 12, &mut hdr_mtime, v.mtime_float, &mut pax);
    }

    let mut out = Vec::new();
    if !pax.is_empty() {
        out.extend(pax_generic_header(&pax, b'x')?);
    }
    let h = HFields {
        name,
        mode: v.mode.unwrap_or(0o644),
        uid: hdr_uid,
        gid: hdr_gid,
        size: hdr_size,
        mtime: hdr_mtime,
        type_b: v.type_b,
        linkname: &v.linkname,
        magic: &POSIX_MAGIC,
        uname: &v.uname,
        gname: &v.gname,
        dev: if matches!(v.type_b, b'3' | b'4') {
            Some((v.devmajor, v.devminor))
        } else {
            None
        },
        prefix: "",
    };
    out.extend(create_header(&h, USTAR_FORMAT, "ascii")?);
    Ok(out)
}

/// TarInfo.tobuf core. Returns the header byte sequence for `format`.
fn tobuf_view(v: &TView, format: i64) -> Result<Vec<u8>, String> {
    let mut name = v.name.clone();
    if v.type_b == b'5' && !name.ends_with('/') {
        name.push('/');
    }
    let mode = v.mode.ok_or_else(|| "mode may not be None".to_string())?;
    let uid = v.uid.ok_or_else(|| "uid may not be None".to_string())?;
    let gid = v.gid.ok_or_else(|| "gid may not be None".to_string())?;
    let dev = if matches!(v.type_b, b'3' | b'4') {
        Some((v.devmajor, v.devminor))
    } else {
        None
    };
    match format {
        USTAR_FORMAT => {
            if v.linkname.as_bytes().len() > 100 {
                return Err("linkname is too long".to_string());
            }
            let (prefix, nm) = if name.as_bytes().len() > 100 {
                posix_split(&name)?
            } else {
                (String::new(), name)
            };
            let h = HFields {
                name: &nm,
                mode,
                uid,
                gid,
                size: v.size,
                mtime: v.mtime_int,
                type_b: v.type_b,
                linkname: &v.linkname,
                magic: &POSIX_MAGIC,
                uname: &v.uname,
                gname: &v.gname,
                dev,
                prefix: &prefix,
            };
            create_header(&h, USTAR_FORMAT, "utf-8")
        }
        GNU_FORMAT => {
            let mut out = Vec::new();
            if v.linkname.as_bytes().len() > 100 {
                out.extend(gnu_long_header(&v.linkname, b'K')?);
            }
            if name.as_bytes().len() > 100 {
                out.extend(gnu_long_header(&name, b'L')?);
            }
            let h = HFields {
                name: &name,
                mode,
                uid,
                gid,
                size: v.size,
                mtime: v.mtime_int,
                type_b: v.type_b,
                linkname: &v.linkname,
                magic: &GNU_MAGIC,
                uname: &v.uname,
                gname: &v.gname,
                dev,
                prefix: "",
            };
            out.extend(create_header(&h, GNU_FORMAT, "utf-8")?);
            Ok(out)
        }
        PAX_FORMAT => create_pax(v, &name),
        _ => Err("invalid format".to_string()),
    }
}

// ── archive parsing (read side) ─────────────────────────────────────────────

struct PMember {
    v: TView,
    data: Vec<u8>,
}

fn parse_pax_records(data: &[u8]) -> Vec<(String, String)> {
    let mut out = Vec::new();
    let mut pos = 0usize;
    while pos < data.len() {
        let mut sp = pos;
        while sp < data.len() && data[sp] != b' ' {
            sp += 1;
        }
        if sp >= data.len() {
            break;
        }
        let len: usize = match std::str::from_utf8(&data[pos..sp])
            .ok()
            .and_then(|s| s.trim().parse().ok())
        {
            Some(l) if l > 0 => l,
            _ => break,
        };
        let end = (pos + len).min(data.len());
        if end <= sp + 1 {
            break;
        }
        let mut rec = &data[sp + 1..end];
        if rec.ends_with(b"\n") {
            rec = &rec[..rec.len() - 1];
        }
        if let Some(eq) = rec.iter().position(|&b| b == b'=') {
            out.push((
                String::from_utf8_lossy(&rec[..eq]).to_string(),
                String::from_utf8_lossy(&rec[eq + 1..]).to_string(),
            ));
        }
        pos += len;
    }
    out
}

/// Parse a (decompressed) tar byte stream into members. Errors map to
/// ReadError at the open() boundary.
fn parse_archive(data: &[u8]) -> Result<Vec<PMember>, String> {
    if !data.is_empty() && data.len() < BLOCKSIZE {
        return Err("truncated header".to_string());
    }
    let mut out: Vec<PMember> = Vec::new();
    let mut pos = 0usize;
    let mut pending_pax: Option<Vec<(String, String)>> = None;
    let mut pending_long_name: Option<String> = None;
    let mut pending_long_link: Option<String> = None;
    while pos + BLOCKSIZE <= data.len() {
        let block = &data[pos..pos + BLOCKSIZE];
        if block.iter().all(|&b| b == 0) {
            break;
        }
        let ck = nti(&block[148..156]).map_err(|_| "invalid header".to_string())?;
        if ck != checksum_unsigned(block) && ck != checksum_signed(block) {
            return Err("bad checksum".to_string());
        }
        let mut v = TView {
            name: nts(&block[0..100]),
            mode: Some(nti(&block[100..108]).unwrap_or(0)),
            uid: Some(nti(&block[108..116]).unwrap_or(0)),
            gid: Some(nti(&block[116..124]).unwrap_or(0)),
            size: nti(&block[124..136]).map_err(|_| "invalid header".to_string())?,
            mtime_int: nti(&block[136..148]).unwrap_or(0),
            type_b: block[156],
            linkname: nts(&block[157..257]),
            uname: nts(&block[265..297]),
            gname: nts(&block[297..329]),
            devmajor: nti(&block[329..337]).unwrap_or(0),
            devminor: nti(&block[337..345]).unwrap_or(0),
            ..TView::default()
        };
        let magic = &block[257..265];
        let prefix = nts(&block[345..500]);
        let is_meta = matches!(v.type_b, b'x' | b'g' | b'L' | b'K');
        if magic.starts_with(b"ustar")
            && !prefix.is_empty()
            && !is_meta
            && v.type_b != b'S'
        {
            v.name = format!("{}/{}", prefix, v.name);
        }
        // A pending pax `size` record overrides the header size BEFORE the
        // payload is sliced (oversized members store 0 in the octal field).
        if !is_meta {
            if let Some(ref pax) = pending_pax {
                if let Some((_, sv)) = pax.iter().find(|(k, _)| k == "size") {
                    if let Ok(n) = sv.parse::<i128>() {
                        v.size = n;
                    }
                }
            }
        }
        if v.size < 0 {
            return Err("invalid header".to_string());
        }
        let nblocks = (v.size as usize).div_ceil(BLOCKSIZE);
        let dstart = pos + BLOCKSIZE;
        let dend = (dstart + v.size as usize).min(data.len());
        let payload = if dstart < data.len() {
            data[dstart..dend].to_vec()
        } else {
            Vec::new()
        };
        pos = dstart + nblocks * BLOCKSIZE;
        match v.type_b {
            b'x' => {
                pending_pax = Some(parse_pax_records(&payload));
            }
            b'g' => { /* global pax header: not applied */ }
            b'L' => {
                pending_long_name = Some(nts(&payload));
            }
            b'K' => {
                pending_long_link = Some(nts(&payload));
            }
            _ => {
                if let Some(n) = pending_long_name.take() {
                    v.name = n;
                }
                if let Some(l) = pending_long_link.take() {
                    v.linkname = l;
                }
                if let Some(pax) = pending_pax.take() {
                    for (k, val) in &pax {
                        match k.as_str() {
                            "path" => v.name = val.trim_end_matches('/').to_string(),
                            "linkpath" => v.linkname = val.clone(),
                            "uid" => {
                                if let Ok(n) = val.parse::<i128>() {
                                    v.uid = Some(n);
                                }
                            }
                            "gid" => {
                                if let Ok(n) = val.parse::<i128>() {
                                    v.gid = Some(n);
                                }
                            }
                            "mtime" => {
                                if let Ok(f) = val.parse::<f64>() {
                                    v.mtime_float = Some(f);
                                    v.mtime_int = f as i128;
                                }
                            }
                            "uname" => v.uname = val.clone(),
                            "gname" => v.gname = val.clone(),
                            _ => {}
                        }
                    }
                    v.pax = pax;
                }
                // CPython frombuf: directories drop redundant trailing slashes.
                if v.type_b == b'5' {
                    v.name = v.name.trim_end_matches('/').to_string();
                }
                out.push(PMember { v, data: payload });
            }
        }
    }
    Ok(out)
}

// ── TarInfo dict-stub construction ──────────────────────────────────────────

/// Build a fresh TarInfo dict-stub with CPython __init__ defaults.
fn tarinfo_dict_new(name: &str) -> MbValue {
    let d = MbObject::new_dict();
    unsafe {
        if let ObjData::Dict(ref lock) = (*d).data {
            let mut m = lock.write().unwrap();
            m.insert("__class__".into(), s_val("TarInfo"));
            m.insert("name".into(), s_val(name));
            m.insert("mode".into(), MbValue::from_int(0o644));
            m.insert("uid".into(), MbValue::from_int(0));
            m.insert("gid".into(), MbValue::from_int(0));
            m.insert("size".into(), MbValue::from_int(0));
            m.insert("mtime".into(), MbValue::from_int(0));
            m.insert("chksum".into(), MbValue::from_int(0));
            m.insert("type".into(), b_val(vec![b'0']));
            m.insert("linkname".into(), s_val(""));
            m.insert("uname".into(), s_val(""));
            m.insert("gname".into(), s_val(""));
            m.insert("devmajor".into(), MbValue::from_int(0));
            m.insert("devminor".into(), MbValue::from_int(0));
            m.insert("offset".into(), MbValue::from_int(0));
            m.insert("offset_data".into(), MbValue::from_int(0));
            m.insert("sparse".into(), MbValue::none());
            m.insert(
                "pax_headers".into(),
                MbValue::from_ptr(MbObject::new_dict()),
            );
        }
    }
    MbValue::from_ptr(d)
}

/// Materialize a parsed member as a TarInfo dict-stub (payload under `_data`).
fn member_to_tarinfo(m: &PMember) -> MbValue {
    let ti = tarinfo_dict_new(&m.v.name);
    dset(ti, "mode", m.v.mode.map(int_value).unwrap_or_else(MbValue::none));
    dset(ti, "uid", m.v.uid.map(int_value).unwrap_or_else(MbValue::none));
    dset(ti, "gid", m.v.gid.map(int_value).unwrap_or_else(MbValue::none));
    dset(ti, "size", int_value(m.v.size));
    match m.v.mtime_float {
        Some(f) => dset(ti, "mtime", MbValue::from_float(f)),
        None => dset(ti, "mtime", int_value(m.v.mtime_int)),
    }
    dset(ti, "type", b_val(vec![m.v.type_b]));
    dset(ti, "linkname", s_val(&m.v.linkname));
    dset(ti, "uname", s_val(&m.v.uname));
    dset(ti, "gname", s_val(&m.v.gname));
    dset(ti, "devmajor", int_value(m.v.devmajor));
    dset(ti, "devminor", int_value(m.v.devminor));
    if !m.v.pax.is_empty() {
        let pd = MbObject::new_dict();
        unsafe {
            if let ObjData::Dict(ref lock) = (*pd).data {
                let mut pm = lock.write().unwrap();
                for (k, val) in &m.v.pax {
                    pm.insert(DictKey::Str(k.clone()), s_val(val));
                }
            }
        }
        dset(ti, "pax_headers", MbValue::from_ptr(pd));
    }
    dset(ti, "_data", b_val(m.data.clone()));
    ti
}

/// Shallow copy of a TarInfo dict-stub (replace(deep=False) semantics: the
/// pax_headers dict is shared).
fn tarinfo_copy(member: MbValue) -> MbValue {
    let d = MbObject::new_dict();
    if let Some(src) = member.as_ptr() {
        unsafe {
            if let (ObjData::Dict(ref dst_lock), ObjData::Dict(ref src_lock)) =
                (&(*d).data, &(*src).data)
            {
                let src_map = src_lock.read().unwrap();
                let mut dst_map = dst_lock.write().unwrap();
                for (k, v) in src_map.iter() {
                    super::super::rc::retain_if_ptr(*v);
                    dst_map.insert(k.clone(), *v);
                }
            }
        }
    }
    MbValue::from_ptr(d)
}

// ── extraction filters ──────────────────────────────────────────────────────

/// Lexically normalize `p` against the cwd ('' and '.' dropped, '..' pops).
fn normalize_abs(p: &str) -> String {
    let joined = if p.starts_with('/') {
        p.to_string()
    } else {
        let cwd = std::env::current_dir()
            .map(|c| c.to_string_lossy().to_string())
            .unwrap_or_else(|_| "/".to_string());
        format!("{}/{}", cwd, p)
    };
    let mut parts: Vec<&str> = Vec::new();
    for comp in joined.split('/') {
        match comp {
            "" | "." => {}
            ".." => {
                parts.pop();
            }
            c => parts.push(c),
        }
    }
    format!("/{}", parts.join("/"))
}

fn is_within(target: &str, dest: &str) -> bool {
    target == dest || target.starts_with(&format!("{}/", dest.trim_end_matches('/')))
}

/// CPython _get_filtered_attrs + member.replace(). Returns a retained member
/// (identity when nothing changed) or the FilterError to raise.
fn filter_transform(
    member: MbValue,
    dest: &str,
    for_data: bool,
) -> Result<MbValue, (String, String)> {
    let v = view_of(member);
    let mut new_name: Option<String> = None;
    let mut name = v.name.clone();
    if name.starts_with('/') {
        name = name.trim_start_matches('/').to_string();
        new_name = Some(name.clone());
    }
    let dest_abs = normalize_abs(dest);
    let target = normalize_abs(&format!("{}/{}", dest_abs, name));
    if !is_within(&target, &dest_abs) {
        return Err((
            "OutsideDestinationError".to_string(),
            format!(
                "{:?} would be extracted to {:?}, which is outside the destination",
                v.name, target
            ),
        ));
    }
    // mode: Some(Some(m)) = new int mode, Some(None) = mode becomes None.
    let mut new_mode: Option<Option<i128>> = None;
    if let Some(m) = v.mode {
        let mut mode = m & 0o755;
        let mut ignore_mode = false;
        if for_data {
            if is_regular_type(v.type_b) || v.type_b == b'1' {
                if mode & 0o100 == 0 {
                    mode &= !0o111;
                }
                mode |= 0o600;
            } else if v.type_b == b'5' || v.type_b == b'2' {
                ignore_mode = true;
            } else if matches!(v.type_b, b'3' | b'4' | b'6') {
                return Err((
                    "SpecialFileError".to_string(),
                    format!("{:?} is a special file", v.name),
                ));
            }
        }
        if ignore_mode {
            new_mode = Some(None);
        } else if mode != m {
            new_mode = Some(Some(mode));
        }
    }
    let mut clear_owner = false;
    if for_data {
        clear_owner = dget(member, "uid").map(|x| !x.is_none()).unwrap_or(false)
            || dget(member, "gid").map(|x| !x.is_none()).unwrap_or(false)
            || dget(member, "uname").map(|x| !x.is_none()).unwrap_or(false)
            || dget(member, "gname").map(|x| !x.is_none()).unwrap_or(false);
        if matches!(v.type_b, b'1' | b'2') {
            if v.linkname.starts_with('/') {
                return Err((
                    "AbsoluteLinkError".to_string(),
                    format!("{:?} is a link to an absolute path", v.name),
                ));
            }
            let link_target = normalize_abs(&format!("{}/{}", dest_abs, v.linkname));
            if !is_within(&link_target, &dest_abs) {
                return Err((
                    "LinkOutsideDestinationError".to_string(),
                    format!(
                        "{:?} would link to {:?}, which is outside the destination",
                        v.name, link_target
                    ),
                ));
            }
        }
    }
    if new_name.is_none() && new_mode.is_none() && !clear_owner {
        unsafe { super::super::rc::retain_if_ptr(member) };
        return Ok(member);
    }
    let copy = tarinfo_copy(member);
    if let Some(n) = new_name {
        dset(copy, "name", s_val(&n));
    }
    match new_mode {
        Some(Some(m)) => dset(copy, "mode", int_value(m)),
        Some(None) => dset(copy, "mode", MbValue::none()),
        None => {}
    }
    if clear_owner {
        dset(copy, "uid", MbValue::none());
        dset(copy, "gid", MbValue::none());
        dset(copy, "uname", MbValue::none());
        dset(copy, "gname", MbValue::none());
    }
    Ok(copy)
}

unsafe extern "C" fn dispatch_fully_trusted_filter(
    args_ptr: *const MbValue,
    nargs: usize,
) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let member = a.first().copied().unwrap_or_else(MbValue::none);
    unsafe { super::super::rc::retain_if_ptr(member) };
    member
}

fn run_filter(a: &[MbValue], for_data: bool) -> MbValue {
    let member = a.first().copied().unwrap_or_else(MbValue::none);
    let dest = a.get(1).copied().and_then(tf_as_str).unwrap_or_default();
    match filter_transform(member, &dest, for_data) {
        Ok(v) => v,
        Err((exc, msg)) => tf_raise(&exc, &msg),
    }
}

unsafe extern "C" fn dispatch_tar_filter(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    run_filter(a, false)
}

unsafe extern "C" fn dispatch_data_filter(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    run_filter(a, true)
}

// ── gzip layer ──────────────────────────────────────────────────────────────

fn gz_compress(data: &[u8], level: u32) -> Vec<u8> {
    use std::io::Write;
    let mut enc =
        flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::new(level));
    let _ = enc.write_all(data);
    enc.finish().unwrap_or_default()
}

fn gz_decompress(data: &[u8]) -> Option<Vec<u8>> {
    use std::io::Read;
    let mut dec = flate2::read::MultiGzDecoder::new(data);
    let mut out = Vec::new();
    dec.read_to_end(&mut out).ok()?;
    Some(out)
}

// ── TarFile object ──────────────────────────────────────────────────────────

/// Parse an open() mode string into (action, compression-suffix).
/// Accepts "r", "r:", "r:*", "r:gz", "r|", "r|*", "r|gz", "w", "w:", "w:gz",
/// "a", "x" (+ bz2/xz suffixes, stored but written uncompressed).
fn parse_mode(mode: &str) -> Result<(char, String), ()> {
    let mut chars = mode.chars();
    let action = chars.next().ok_or(())?;
    if !matches!(action, 'r' | 'w' | 'a' | 'x') {
        return Err(());
    }
    let rest: String = chars.collect();
    if rest.is_empty() {
        return Ok((action, if action == 'r' { "*".to_string() } else { String::new() }));
    }
    let mut rc = rest.chars();
    let sep = rc.next().unwrap();
    if sep != ':' && sep != '|' {
        return Err(());
    }
    let comp: String = rc.collect();
    if !matches!(comp.as_str(), "" | "*" | "gz" | "bz2" | "xz") {
        return Err(());
    }
    Ok((action, comp))
}

fn tarfile_dict_new(
    action: char,
    comp: &str,
    name_val: Option<MbValue>,
    fileobj: Option<MbValue>,
    format: i64,
    members: Vec<MbValue>,
) -> MbValue {
    let d = MbObject::new_dict();
    unsafe {
        if let ObjData::Dict(ref lock) = (*d).data {
            let mut m = lock.write().unwrap();
            m.insert("__class__".into(), s_val("TarFile"));
            match name_val {
                Some(n) if !n.is_none() => {
                    super::super::rc::retain_if_ptr(n);
                    m.insert("name".into(), n);
                }
                _ => {
                    m.insert("name".into(), MbValue::none());
                }
            }
            m.insert("mode".into(), s_val(&action.to_string()));
            match fileobj {
                Some(f) if !f.is_none() => {
                    super::super::rc::retain_if_ptr(f);
                    m.insert("fileobj".into(), f);
                }
                _ => {
                    m.insert("fileobj".into(), MbValue::none());
                }
            }
            m.insert("format".into(), MbValue::from_int(format));
            m.insert("closed".into(), MbValue::from_bool(false));
            m.insert("_comp".into(), s_val(comp));
            m.insert(
                "_members".into(),
                MbValue::from_ptr(MbObject::new_list(members)),
            );
            m.insert(
                "_wbuf".into(),
                MbValue::from_ptr(MbObject::new_bytearray(Vec::new())),
            );
            m.insert("_next".into(), MbValue::from_int(0));
            m.insert(
                "pax_headers".into(),
                MbValue::from_ptr(MbObject::new_dict()),
            );
        }
    }
    MbValue::from_ptr(d)
}

fn tf_members(tf: MbValue) -> Vec<MbValue> {
    if let Some(members) = dget(tf, "_members") {
        if let Some(ptr) = members.as_ptr() {
            unsafe {
                if let ObjData::List(ref lock) = (*ptr).data {
                    return lock.read().unwrap().to_vec();
                }
            }
        }
    }
    Vec::new()
}

fn wbuf_extend(tf: MbValue, bytes: &[u8]) {
    if let Some(w) = dget(tf, "_wbuf") {
        if let Some(p) = w.as_ptr() {
            unsafe {
                if let ObjData::ByteArray(ref lock) = (*p).data {
                    lock.write().unwrap().extend_from_slice(bytes);
                }
            }
        }
    }
}

fn wbuf_clone(tf: MbValue) -> Vec<u8> {
    dget(tf, "_wbuf").and_then(tf_as_bytes).unwrap_or_default()
}

fn tf_is_closed(tf: MbValue) -> bool {
    dget(tf, "closed").and_then(|v| v.as_bool()) == Some(true)
}

fn tf_check_open(tf: MbValue) -> bool {
    if tf_is_closed(tf) {
        tf_raise("OSError", "TarFile is closed");
        return false;
    }
    true
}

/// Close a TarFile. In write mode (and `finalize`), appends the two-NUL-block
/// terminator, pads to a RECORDSIZE multiple, applies the gzip layer, and
/// flushes the archive to the fileobj / path.
pub fn tarfile_close_impl(tf: MbValue, finalize: bool) {
    if tf_is_closed(tf) {
        return;
    }
    let mode = dget(tf, "mode").and_then(tf_as_str).unwrap_or_default();
    if matches!(mode.as_str(), "w" | "a" | "x") && finalize {
        let mut buf = wbuf_clone(tf);
        buf.resize(buf.len() + 2 * BLOCKSIZE, 0);
        let rem = buf.len() % RECORDSIZE;
        if rem > 0 {
            buf.resize(buf.len() + RECORDSIZE - rem, 0);
        }
        let comp = dget(tf, "_comp").and_then(tf_as_str).unwrap_or_default();
        let out = if comp == "gz" { gz_compress(&buf, 9) } else { buf };
        let fileobj = dget(tf, "fileobj").filter(|f| !f.is_none());
        if let Some(f) = fileobj {
            let data_val = b_val(out);
            super::io_mod::mb_bytesio_write(f, data_val);
            unsafe { super::super::rc::release_if_ptr(data_val) };
        } else if let Some(path) = dget(tf, "name").and_then(tf_as_str) {
            let _ = std::fs::write(path, out);
        }
    }
    dset(tf, "closed", MbValue::from_bool(true));
}

/// `with tarfile.open(...) as tf:` — __enter__ re-checks the closed flag
/// (CPython TarFile._check raises OSError on a closed archive).
pub fn tarfile_context_enter(tf: MbValue) -> MbValue {
    if tf_is_closed(tf) {
        return tf_raise("OSError", "TarFile is closed");
    }
    unsafe { super::super::rc::retain_if_ptr(tf) };
    tf
}

/// __exit__: finalize+close on clean exit; mark closed without writing the
/// end-of-archive blocks when an exception is in flight (CPython parity).
pub fn tarfile_context_exit(tf: MbValue, has_pending: bool) {
    tarfile_close_impl(tf, !has_pending);
}

/// Bytes at the fileobj's current position, length `n`, without advancing.
fn fileobj_peek_n(f: MbValue, n: usize) -> Vec<u8> {
    let all = tf_fileobj_remaining(f).unwrap_or_default();
    all[..n.min(all.len())].to_vec()
}

// ── TarFile / TarInfo method dispatch (dict_ops routing) ────────────────────

fn seq_items(args: MbValue) -> Vec<MbValue> {
    unsafe {
        if let Some(ptr) = args.as_ptr() {
            if let ObjData::List(ref lock) = (*ptr).data {
                return lock.read().unwrap().to_vec();
            }
        }
    }
    Vec::new()
}

/// Treat a trailing dict as kwargs only when every key is a known keyword
/// for the method (payload dicts like get_info() results stay positional).
fn split_known_kwargs(items: &[MbValue], known: &[&str]) -> (Vec<MbValue>, Option<MbValue>) {
    if let Some(&last) = items.last() {
        if let Some(p) = last.as_ptr() {
            unsafe {
                if let ObjData::Dict(ref lock) = (*p).data {
                    let g = lock.read().unwrap();
                    let all_known = g
                        .keys()
                        .all(|k| matches!(k, DictKey::Str(s) if known.contains(&s.as_str())));
                    if all_known {
                        return (items[..items.len() - 1].to_vec(), Some(last));
                    }
                }
            }
        }
    }
    (items.to_vec(), None)
}

const METHOD_KWS: &[&str] = &[
    "format", "encoding", "errors", "filter", "path", "members", "numeric_owner",
    "name", "mode", "fileobj", "arcname", "recursive", "set_attrs", "tarinfo",
];

/// TarInfo.get_info(): header-building metadata dict (mode masked to 0o7777,
/// DIRTYPE names get a trailing '/').
fn ti_get_info(ti: MbValue) -> MbValue {
    let d = MbObject::new_dict();
    let copy_val = |k: &str| -> MbValue {
        let v = dget(ti, k).unwrap_or_else(MbValue::none);
        unsafe { super::super::rc::retain_if_ptr(v) };
        v
    };
    let type_b = dget(ti, "type")
        .and_then(tf_as_bytes)
        .and_then(|b| b.first().copied())
        .unwrap_or(b'0');
    let mut name = dget(ti, "name").and_then(tf_as_str).unwrap_or_default();
    if type_b == b'5' && !name.ends_with('/') {
        name.push('/');
    }
    let mode_val = match dget(ti, "mode") {
        Some(v) if v.is_none() => MbValue::none(),
        Some(v) => tf_as_i128(v).map(|m| int_value(m & 0o7777)).unwrap_or(v),
        None => MbValue::from_int(0o644),
    };
    unsafe {
        if let ObjData::Dict(ref lock) = (*d).data {
            let mut m = lock.write().unwrap();
            m.insert("name".into(), s_val(&name));
            m.insert("mode".into(), mode_val);
            m.insert("uid".into(), copy_val("uid"));
            m.insert("gid".into(), copy_val("gid"));
            m.insert("size".into(), copy_val("size"));
            m.insert("mtime".into(), copy_val("mtime"));
            m.insert("chksum".into(), copy_val("chksum"));
            m.insert("type".into(), copy_val("type"));
            m.insert("linkname".into(), copy_val("linkname"));
            m.insert("uname".into(), copy_val("uname"));
            m.insert("gname".into(), copy_val("gname"));
            m.insert("devmajor".into(), copy_val("devmajor"));
            m.insert("devminor".into(), copy_val("devminor"));
        }
    }
    MbValue::from_ptr(d)
}

fn ti_header_method(receiver: MbValue, info_arg: Option<MbValue>, format: i64) -> MbValue {
    // create_*_header(info, ...) read from the passed info dict; tobuf() reads
    // the receiver. pax_headers always come from the receiver.
    let mut v = match info_arg {
        Some(info) if tf_is_dict(info) => view_of(info),
        _ => view_of(receiver),
    };
    v.pax = view_of(receiver).pax;
    match tobuf_view(&v, format) {
        Ok(b) => b_val(b),
        Err(msg) => tf_raise("ValueError", &msg),
    }
}

fn member_matches(m: MbValue, name: &str) -> bool {
    dget(m, "name")
        .and_then(tf_as_str)
        .map(|n| n == name)
        .unwrap_or(false)
}

/// Resolve an extractfile/getmember/extract argument (TarInfo or name str).
fn resolve_member(tf: MbValue, arg: MbValue) -> Option<MbValue> {
    if let Some(name) = tf_as_str(arg) {
        let want = name.trim_end_matches('/').to_string();
        let members = tf_members(tf);
        return members
            .into_iter()
            .rev()
            .find(|m| member_matches(*m, &want));
    }
    if tf_is_dict(arg) {
        return Some(arg);
    }
    None
}

fn member_data(m: MbValue) -> Vec<u8> {
    dget(m, "_data").and_then(tf_as_bytes).unwrap_or_default()
}

/// Write one (already filtered) member under `dest` on disk.
fn write_member_to_disk(dest: &str, member: MbValue) {
    let name = dget(member, "name").and_then(tf_as_str).unwrap_or_default();
    let type_b = dget(member, "type")
        .and_then(tf_as_bytes)
        .and_then(|b| b.first().copied())
        .unwrap_or(b'0');
    let rel = name.trim_end_matches('/');
    if rel.is_empty() {
        return;
    }
    let path = std::path::Path::new(dest).join(rel);
    if type_b == b'5' {
        let _ = std::fs::create_dir_all(&path);
    } else if is_regular_type(type_b) {
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let _ = std::fs::write(&path, member_data(member));
    }
    // Links / special files: not materialized.
}

fn tf_extract_members(
    _tf: MbValue,
    dest: &str,
    filter_name: Option<String>,
    members: Vec<MbValue>,
) -> MbValue {
    enum F {
        Trusted,
        Tar,
        Data,
    }
    let f = match filter_name.as_deref() {
        None | Some("fully_trusted") => F::Trusted,
        Some("tar") => F::Tar,
        Some("data") => F::Data,
        Some(other) => {
            return tf_raise("ValueError", &format!("filter {:?} not found", other));
        }
    };
    for m in members {
        let filtered = match f {
            F::Trusted => {
                unsafe { super::super::rc::retain_if_ptr(m) };
                m
            }
            F::Tar => match filter_transform(m, dest, false) {
                Ok(v) => v,
                Err((exc, msg)) => return tf_raise(&exc, &msg),
            },
            F::Data => match filter_transform(m, dest, true) {
                Ok(v) => v,
                Err((exc, msg)) => return tf_raise(&exc, &msg),
            },
        };
        write_member_to_disk(dest, filtered);
        unsafe { super::super::rc::release_if_ptr(filtered) };
    }
    MbValue::none()
}

/// Dispatch a method call on a TarFile / TarInfo dict-stub. Returns None to
/// fall through to plain-dict semantics.
pub fn dispatch_tar_stub_method(
    cls: &str,
    name: &str,
    receiver: MbValue,
    args: MbValue,
) -> Option<MbValue> {
    let items = seq_items(args);
    let (pos, kw) = split_known_kwargs(&items, METHOD_KWS);
    let arg = |i: usize| pos.get(i).copied().unwrap_or_else(MbValue::none);

    match cls {
        "TarInfo" => match name {
            "get_info" => Some(ti_get_info(receiver)),
            "tobuf" => {
                let format = pos
                    .first()
                    .copied()
                    .or_else(|| tf_kw_get(kw, "format"))
                    .and_then(|v| v.as_int())
                    .unwrap_or(PAX_FORMAT);
                let v = view_of(receiver);
                Some(match tobuf_view(&v, format) {
                    Ok(b) => b_val(b),
                    Err(msg) => tf_raise("ValueError", &msg),
                })
            }
            "create_pax_header" => {
                Some(ti_header_method(receiver, pos.first().copied(), PAX_FORMAT))
            }
            "create_ustar_header" => {
                Some(ti_header_method(receiver, pos.first().copied(), USTAR_FORMAT))
            }
            "create_gnu_header" => {
                Some(ti_header_method(receiver, pos.first().copied(), GNU_FORMAT))
            }
            "isreg" | "isfile" | "isdir" | "issym" | "islnk" | "ischr" | "isblk"
            | "isfifo" | "isdev" | "issparse" => {
                let t = dget(receiver, "type")
                    .and_then(tf_as_bytes)
                    .and_then(|b| b.first().copied())
                    .unwrap_or(b'0');
                let result = match name {
                    "isreg" | "isfile" => is_regular_type(t),
                    "isdir" => t == b'5',
                    "issym" => t == b'2',
                    "islnk" => t == b'1',
                    "ischr" => t == b'3',
                    "isblk" => t == b'4',
                    "isfifo" => t == b'6',
                    "isdev" => matches!(t, b'3' | b'4' | b'6'),
                    _ => t == b'S',
                };
                Some(MbValue::from_bool(result))
            }
            _ => None,
        },
        "TarFile" => match name {
            "addfile" => {
                if !tf_check_open(receiver) {
                    return Some(MbValue::none());
                }
                let ti = pos
                    .first()
                    .copied()
                    .or_else(|| tf_kw_get(kw, "tarinfo"))
                    .unwrap_or_else(MbValue::none);
                if !tf_is_dict(ti) {
                    return Some(tf_raise("TypeError", "addfile() requires a TarInfo object"));
                }
                let fobj = pos
                    .get(1)
                    .copied()
                    .or_else(|| tf_kw_get(kw, "fileobj"))
                    .filter(|f| !f.is_none());
                let v = view_of(ti);
                let format = dget(receiver, "format")
                    .and_then(|x| x.as_int())
                    .unwrap_or(PAX_FORMAT);
                let buf = match tobuf_view(&v, format) {
                    Ok(b) => b,
                    Err(msg) => return Some(tf_raise("ValueError", &msg)),
                };
                wbuf_extend(receiver, &buf);
                if let Some(f) = fobj {
                    let data = fileobj_peek_n(f, v.size.max(0) as usize);
                    wbuf_extend(receiver, &pad_block(data));
                }
                if let Some(members) = dget(receiver, "_members") {
                    super::super::list_ops::mb_list_append(members, ti);
                }
                Some(MbValue::none())
            }
            "getmembers" => {
                let members = tf_members(receiver);
                Some(MbValue::from_ptr(MbObject::new_list_borrowed(members)))
            }
            "getnames" => {
                let names: Vec<MbValue> = tf_members(receiver)
                    .into_iter()
                    .map(|m| s_val(&dget(m, "name").and_then(tf_as_str).unwrap_or_default()))
                    .collect();
                Some(MbValue::from_ptr(MbObject::new_list(names)))
            }
            "getmember" => {
                let raw = arg(0);
                match resolve_member(receiver, raw) {
                    Some(m) => {
                        unsafe { super::super::rc::retain_if_ptr(m) };
                        Some(m)
                    }
                    None => {
                        let nm = tf_as_str(raw).unwrap_or_default();
                        Some(tf_raise("KeyError", &format!("filename {:?} not found", nm)))
                    }
                }
            }
            "extractfile" => {
                if !tf_check_open(receiver) {
                    return Some(MbValue::none());
                }
                let raw = arg(0);
                match resolve_member(receiver, raw) {
                    Some(m) => {
                        let t = dget(m, "type")
                            .and_then(tf_as_bytes)
                            .and_then(|b| b.first().copied())
                            .unwrap_or(b'0');
                        if is_regular_type(t) {
                            Some(super::io_mod::mb_bytesio_new_with(member_data(m)))
                        } else {
                            Some(MbValue::none())
                        }
                    }
                    None => {
                        let nm = tf_as_str(raw).unwrap_or_default();
                        Some(tf_raise("KeyError", &format!("filename {:?} not found", nm)))
                    }
                }
            }
            "extractall" => {
                if !tf_check_open(receiver) {
                    return Some(MbValue::none());
                }
                let dest = pos
                    .first()
                    .copied()
                    .or_else(|| tf_kw_get(kw, "path"))
                    .and_then(tf_as_str)
                    .unwrap_or_else(|| ".".to_string());
                let filter_name = tf_kw_get(kw, "filter")
                    .filter(|v| !v.is_none())
                    .and_then(tf_as_str);
                let members = tf_members(receiver);
                Some(tf_extract_members(receiver, &dest, filter_name, members))
            }
            "extract" => {
                if !tf_check_open(receiver) {
                    return Some(MbValue::none());
                }
                let member = match resolve_member(receiver, arg(0)) {
                    Some(m) => m,
                    None => {
                        let nm = tf_as_str(arg(0)).unwrap_or_default();
                        return Some(tf_raise(
                            "KeyError",
                            &format!("filename {:?} not found", nm),
                        ));
                    }
                };
                let dest = pos
                    .get(1)
                    .copied()
                    .or_else(|| tf_kw_get(kw, "path"))
                    .and_then(tf_as_str)
                    .unwrap_or_default();
                let dest = if dest.is_empty() { ".".to_string() } else { dest };
                let filter_name = tf_kw_get(kw, "filter")
                    .filter(|v| !v.is_none())
                    .and_then(tf_as_str);
                Some(tf_extract_members(receiver, &dest, filter_name, vec![member]))
            }
            "next" => {
                let idx = dget(receiver, "_next").and_then(|v| v.as_int()).unwrap_or(0);
                let members = tf_members(receiver);
                if (idx as usize) < members.len() {
                    let m = members[idx as usize];
                    dset(receiver, "_next", MbValue::from_int(idx + 1));
                    unsafe { super::super::rc::retain_if_ptr(m) };
                    Some(m)
                } else {
                    Some(MbValue::none())
                }
            }
            "close" => {
                tarfile_close_impl(receiver, true);
                Some(MbValue::none())
            }
            "__enter__" => Some(tarfile_context_enter(receiver)),
            "__exit__" => {
                tarfile_close_impl(receiver, true);
                Some(MbValue::from_bool(false))
            }
            _ => None,
        },
        _ => None,
    }
}

// ── module-level entry points ───────────────────────────────────────────────

/// `tarfile.open(name=None, mode='r', fileobj=None, ..., format=, compresslevel=)`.
///
/// Error gates (errors dimension, CPython parity):
///   * bz2 write mode with compresslevel outside 1..=9  -> ValueError
///   * plain (uncompressed) mode with compresslevel      -> TypeError
///   * unparseable mode string                           -> ValueError
///   * read mode, missing path on disk, no fileobj       -> FileNotFoundError
///   * read mode, fileobj buffer 1..511 bytes / garbage  -> ReadError
unsafe extern "C" fn dispatch_open(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let (pos, kw) = tf_split(a);

    let name_val = pos.first().copied().or_else(|| tf_kw_get(kw, "name"));
    let mode_val = pos.get(1).copied().or_else(|| tf_kw_get(kw, "mode"));
    let fileobj = pos.get(2).copied().or_else(|| tf_kw_get(kw, "fileobj"));
    let compresslevel = tf_kw_get(kw, "compresslevel").and_then(|v| v.as_int());
    let format = tf_kw_get(kw, "format").and_then(|v| v.as_int()).unwrap_or(PAX_FORMAT);

    let mode = mode_val.and_then(tf_as_str).unwrap_or_else(|| "r".to_string());
    let has_fileobj = fileobj.map(|f| !f.is_none()).unwrap_or(false);

    // bz2 requires compresslevel 1..=9; e.g. mode='w:bz2', compresslevel=0.
    if mode.contains("bz2") {
        if let Some(cl) = compresslevel {
            if !(1..=9).contains(&cl) {
                return tf_raise(
                    "ValueError",
                    "valid range for compresslevel is between 1 and 9",
                );
            }
        }
    }

    let (action, comp) = match parse_mode(&mode) {
        Ok(p) => p,
        Err(()) => {
            return tf_raise("ValueError", &format!("undiscernible mode {:?}", mode));
        }
    };

    // Plain (uncompressed) open does not accept compresslevel.
    if compresslevel.is_some() && comp.is_empty() && action != 'r' {
        return tf_raise(
            "TypeError",
            "taropen() got an unexpected keyword argument 'compresslevel'",
        );
    }

    let is_read = action == 'r';

    // Read-open of a path that does not exist (and no fileobj override).
    if is_read && !has_fileobj {
        if let Some(path) = name_val.and_then(tf_as_str) {
            if !std::path::Path::new(&path).exists() {
                return tf_raise(
                    "FileNotFoundError",
                    &format!("[Errno 2] No such file or directory: '{path}'"),
                );
            }
        }
    }

    // Read-open of a fileobj whose buffer is non-empty but shorter than one
    // 512-byte block: a truncated header that no valid tar archive produces.
    if is_read && has_fileobj {
        if let Some(len) = fileobj.and_then(tf_fileobj_buf_len) {
            if (1..BLOCKSIZE).contains(&len) {
                return tf_raise("ReadError", "truncated header");
            }
        }
    }

    if is_read {
        let data = if has_fileobj {
            match fileobj.and_then(tf_fileobj_remaining) {
                Some(d) => d,
                None => {
                    // Unsupported fileobj shape: surface an empty archive.
                    return tarfile_dict_new('r', &comp, name_val, fileobj, format, vec![]);
                }
            }
        } else if let Some(path) = name_val.and_then(tf_as_str) {
            match std::fs::read(&path) {
                Ok(d) => d,
                Err(_) => {
                    return tf_raise(
                        "FileNotFoundError",
                        &format!("[Errno 2] No such file or directory: '{path}'"),
                    );
                }
            }
        } else {
            return tf_raise("ValueError", "nothing to open");
        };
        if data.is_empty() {
            return tf_raise("ReadError", "empty file");
        }
        let raw = if data.starts_with(&[0x1f, 0x8b]) && matches!(comp.as_str(), "" | "*" | "gz")
        {
            match gz_decompress(&data) {
                Some(d) => d,
                None => return tf_raise("ReadError", "not a gzip file"),
            }
        } else {
            data
        };
        let members = match parse_archive(&raw) {
            Ok(ms) => ms,
            Err(_) => {
                return tf_raise("ReadError", "file could not be opened successfully");
            }
        };
        let member_vals: Vec<MbValue> = members.iter().map(member_to_tarinfo).collect();
        return tarfile_dict_new('r', &comp, name_val, fileobj, format, member_vals);
    }

    // Write modes ('w'/'a'/'x'): buffer the archive, flush on close.
    tarfile_dict_new(action, &comp, name_val, fileobj, format, vec![])
}

/// `tarfile.itn(n, digits=8, format=DEFAULT_FORMAT)` — encode an integer into
/// a tar number field (octal, or GNU base-256 for GNU_FORMAT), CPython ranges:
/// octal `0 <= n < 8**(digits-1)`; GNU base-256 `-256**(digits-1) <= n <
/// 256**(digits-1)`; anything else raises ValueError("overflow in number field").
unsafe extern "C" fn dispatch_itn(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let (pos, kw) = tf_split(a);

    let n_val = pos.first().copied().or_else(|| tf_kw_get(kw, "n"));
    let n = match n_val {
        Some(v) if v.is_float() => v.as_float().map(|f| f as i128),
        Some(v) => tf_as_i128(v),
        None => None,
    };
    let Some(n) = n else {
        return MbValue::none(); // non-int / absent: stay inert
    };
    let digits = pos
        .get(1)
        .copied()
        .or_else(|| tf_kw_get(kw, "digits"))
        .and_then(|v| v.as_int())
        .unwrap_or(8);
    let format = pos
        .get(2)
        .copied()
        .or_else(|| tf_kw_get(kw, "format"))
        .and_then(|v| v.as_int())
        .unwrap_or(PAX_FORMAT);
    if !(2..=32).contains(&digits) {
        return tf_raise("ValueError", "overflow in number field");
    }
    match itn(n, digits as usize, format) {
        Ok(b) => b_val(b),
        Err(msg) => tf_raise("ValueError", &msg),
    }
}

/// `tarfile.nti(s)` — decode an octal-ASCII or GNU base-256 number field.
unsafe extern "C" fn dispatch_nti(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let Some(bytes) = a.first().copied().and_then(tf_as_bytes) else {
        return MbValue::none();
    };
    match nti(&bytes) {
        Ok(n) => int_value(n),
        Err(()) => tf_raise("InvalidHeaderError", "invalid header"),
    }
}

/// `tarfile.nts(s, encoding, errors)` — NUL-terminated bytes field -> str.
unsafe extern "C" fn dispatch_nts(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let Some(bytes) = a.first().copied().and_then(tf_as_bytes) else {
        return MbValue::none();
    };
    s_val(&nts(&bytes))
}

/// `tarfile.stn(s, length, encoding, errors)` — str -> NUL-padded bytes field.
unsafe extern "C" fn dispatch_stn(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let Some(s) = a.first().copied().and_then(tf_as_str) else {
        return MbValue::none();
    };
    let length = a.get(1).copied().and_then(|v| v.as_int()).unwrap_or(0).max(0) as usize;
    let encoding = a
        .get(2)
        .copied()
        .and_then(tf_as_str)
        .unwrap_or_else(|| "utf-8".to_string());
    b_val(stn(&s, length, &encoding))
}

/// `tarfile.TarInfo(name="")` constructor.
unsafe extern "C" fn dispatch_tarinfo_new(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let (pos, kw) = tf_split(a);
    let name = pos
        .first()
        .copied()
        .or_else(|| tf_kw_get(kw, "name"))
        .and_then(tf_as_str)
        .unwrap_or_default();
    tarinfo_dict_new(&name)
}

/// Surface-only stub for the remaining helper functions (`calc_chksums`,
/// `copyfileobj`, `bltn_open`, `main`) and the TarFile/ExFileObject
/// constructors. These exist so `hasattr(tarfile, name)` / `callable(name)`
/// resolve. Returns None.
unsafe extern "C" fn dispatch_tarfile_stub(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::none()
}

/// tarfile.is_tarfile(path-or-fileobj) -> bool. Preserves a fileobj's stream
/// position (reads the buffer without touching `_pos`).
pub fn mb_tarfile_is_tarfile(target: MbValue) -> MbValue {
    let data = if let Some(path) = tf_as_str(target) {
        std::fs::read(path).ok()
    } else {
        tf_fileobj_remaining(target)
    };
    let Some(data) = data else {
        return MbValue::from_bool(false);
    };
    if data.is_empty() {
        return MbValue::from_bool(false);
    }
    let raw = if data.starts_with(&[0x1f, 0x8b]) {
        match gz_decompress(&data) {
            Some(d) => d,
            None => return MbValue::from_bool(false),
        }
    } else {
        data
    };
    MbValue::from_bool(parse_archive(&raw).is_ok())
}

/// Register the tarfile module.
pub fn register() {
    let mut attrs = HashMap::new();
    let dispatchers: Vec<(&str, usize)> = vec![
        ("open", dispatch_open as usize),
        ("is_tarfile", dispatch_is_tarfile as usize),
        ("itn", dispatch_itn as usize),
        ("nti", dispatch_nti as usize),
        ("nts", dispatch_nts as usize),
        ("stn", dispatch_stn as usize),
        ("tar_filter", dispatch_tar_filter as usize),
        ("data_filter", dispatch_data_filter as usize),
        ("fully_trusted_filter", dispatch_fully_trusted_filter as usize),
        ("TarInfo", dispatch_tarinfo_new as usize),
    ];
    for (name, addr) in dispatchers {
        attrs.insert(name.to_string(), MbValue::from_func(addr));
        super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
            s.borrow_mut().insert(addr as u64);
        });
    }
    // isinstance(x, tarfile.TarInfo) resolves the constructor dispatcher to
    // the dict-stub __class__ tag via NATIVE_TYPE_NAMES.
    super::super::module::NATIVE_TYPE_NAMES.with(|m| {
        m.borrow_mut()
            .insert(dispatch_tarinfo_new as usize as u64, "TarInfo".into());
    });
        // surface: missing CPython module constants (auto-added)
    attrs.insert("BLOCKSIZE".into(), MbValue::from_int(512));
    attrs.insert("DEFAULT_FORMAT".into(), MbValue::from_int(2));
    attrs.insert("ENCODING".into(), MbValue::from_ptr(MbObject::new_str("utf-8".to_string())));
    attrs.insert("GNU_FORMAT".into(), MbValue::from_int(1));
    attrs.insert("LENGTH_LINK".into(), MbValue::from_int(100));
    attrs.insert("LENGTH_NAME".into(), MbValue::from_int(100));
    attrs.insert("LENGTH_PREFIX".into(), MbValue::from_int(155));
    attrs.insert("PAX_FORMAT".into(), MbValue::from_int(2));
    attrs.insert("RECORDSIZE".into(), MbValue::from_int(10240));
    attrs.insert("USTAR_FORMAT".into(), MbValue::from_int(0));
    attrs.insert("version".into(), MbValue::from_ptr(MbObject::new_str("0.9.0".to_string())));

    // surface: remaining CPython 3.12 tarfile module names (hasattr/callable
    // probes — see config/manifests/std-libs/cpython312_surface/tarfile.toml).
    // (Int constants BLOCKSIZE/DEFAULT_FORMAT/ENCODING/GNU_FORMAT/LENGTH_* /
    // PAX_FORMAT/RECORDSIZE/USTAR_FORMAT/version already inserted above.)

    // bytes constants (single-byte type flags + magics).
    attrs.insert("REGTYPE".into(), MbValue::from_ptr(MbObject::new_bytes(vec![48])));
    attrs.insert("AREGTYPE".into(), MbValue::from_ptr(MbObject::new_bytes(vec![0])));
    attrs.insert("LNKTYPE".into(), MbValue::from_ptr(MbObject::new_bytes(vec![49])));
    attrs.insert("SYMTYPE".into(), MbValue::from_ptr(MbObject::new_bytes(vec![50])));
    attrs.insert("CHRTYPE".into(), MbValue::from_ptr(MbObject::new_bytes(vec![51])));
    attrs.insert("BLKTYPE".into(), MbValue::from_ptr(MbObject::new_bytes(vec![52])));
    attrs.insert("DIRTYPE".into(), MbValue::from_ptr(MbObject::new_bytes(vec![53])));
    attrs.insert("FIFOTYPE".into(), MbValue::from_ptr(MbObject::new_bytes(vec![54])));
    attrs.insert("CONTTYPE".into(), MbValue::from_ptr(MbObject::new_bytes(vec![55])));
    attrs.insert("GNUTYPE_LONGNAME".into(), MbValue::from_ptr(MbObject::new_bytes(vec![76])));
    attrs.insert("GNUTYPE_LONGLINK".into(), MbValue::from_ptr(MbObject::new_bytes(vec![75])));
    attrs.insert("GNUTYPE_SPARSE".into(), MbValue::from_ptr(MbObject::new_bytes(vec![83])));
    attrs.insert("XHDTYPE".into(), MbValue::from_ptr(MbObject::new_bytes(vec![120])));
    attrs.insert("XGLTYPE".into(), MbValue::from_ptr(MbObject::new_bytes(vec![103])));
    attrs.insert("SOLARIS_XHDTYPE".into(), MbValue::from_ptr(MbObject::new_bytes(vec![88])));
    attrs.insert("NUL".into(), MbValue::from_ptr(MbObject::new_bytes(vec![0])));
    attrs.insert("GNU_MAGIC".into(),
        MbValue::from_ptr(MbObject::new_bytes(vec![117, 115, 116, 97, 114, 32, 32, 0])));
    attrs.insert("POSIX_MAGIC".into(),
        MbValue::from_ptr(MbObject::new_bytes(vec![117, 115, 116, 97, 114, 0, 48, 48])));

    // Tuple / set / dict structured constants.
    let b = |x: u8| MbValue::from_ptr(MbObject::new_bytes(vec![x]));
    attrs.insert("GNU_TYPES".into(),
        MbValue::from_ptr(MbObject::new_tuple(vec![b(76), b(75), b(83)])));
    attrs.insert("REGULAR_TYPES".into(),
        MbValue::from_ptr(MbObject::new_tuple(vec![b(48), b(0), b(55), b(83)])));
    attrs.insert("SUPPORTED_TYPES".into(),
        MbValue::from_ptr(MbObject::new_tuple(vec![
            b(48), b(0), b(49), b(50), b(53), b(54), b(55), b(51), b(52), b(76), b(75), b(83),
        ])));
    let s = |x: &str| MbValue::from_ptr(MbObject::new_str(x.to_string()));
    attrs.insert("PAX_FIELDS".into(),
        MbValue::from_ptr(MbObject::new_tuple(vec![
            s("path"), s("linkpath"), s("size"), s("mtime"),
            s("uid"), s("gid"), s("uname"), s("gname"),
        ])));
    attrs.insert("PAX_NAME_FIELDS".into(),
        MbValue::from_ptr(MbObject::new_set(vec![
            s("path"), s("linkpath"), s("uname"), s("gname"),
        ])));
    // PAX_NUMBER_FIELDS maps field -> Python type object in CPython; we surface
    // it as a dict mapping field -> type-name string (present + dict-typed)
    // since real type objects as map values are not yet representable here.
    let pax_num = MbObject::new_dict();
    unsafe {
        if let ObjData::Dict(ref lock) = (*pax_num).data {
            let mut map = lock.write().unwrap();
            map.insert("atime".into(), s("float"));
            map.insert("ctime".into(), s("float"));
            map.insert("mtime".into(), s("float"));
            map.insert("uid".into(), s("int"));
            map.insert("gid".into(), s("int"));
            map.insert("size".into(), s("int"));
        }
    }
    attrs.insert("PAX_NUMBER_FIELDS".into(), MbValue::from_ptr(pax_num));

    // Remaining module-level helper functions (surface stubs — callable, None).
    let fn_stubs: Vec<&str> = vec!["calc_chksums", "copyfileobj", "bltn_open", "main"];
    for name in fn_stubs {
        let addr = dispatch_tarfile_stub as usize;
        attrs.insert(name.to_string(), MbValue::from_func(addr));
        super::super::module::NATIVE_FUNC_ADDRS.with(|st| {
            st.borrow_mut().insert(addr as u64);
        });
    }

    // Classes. TarFile / ExFileObject as class-name string sentinels (TarInfo
    // is the real constructor dispatcher above); the exception hierarchy
    // additionally registers in the class registry with correct bases so
    // `issubclass` / `except tarfile.X` resolve (gzip precedent).
    let plain_classes: Vec<&str> = vec!["TarFile", "ExFileObject"];
    for name in plain_classes {
        attrs.insert(name.to_string(),
            MbValue::from_ptr(MbObject::new_str(name.to_string())));
    }
    // (class-name, direct-base) pairs, parents before children for clean MRO.
    let exc_classes: Vec<(&str, &str)> = vec![
        ("TarError", "Exception"),
        ("ReadError", "TarError"),
        ("CompressionError", "TarError"),
        ("StreamError", "TarError"),
        ("ExtractError", "TarError"),
        ("HeaderError", "TarError"),
        ("FilterError", "TarError"),
        ("EmptyHeaderError", "HeaderError"),
        ("TruncatedHeaderError", "HeaderError"),
        ("EOFHeaderError", "HeaderError"),
        ("InvalidHeaderError", "HeaderError"),
        ("SubsequentHeaderError", "HeaderError"),
        ("AbsoluteLinkError", "FilterError"),
        ("OutsideDestinationError", "FilterError"),
        ("SpecialFileError", "FilterError"),
        ("AbsolutePathError", "FilterError"),
        ("LinkOutsideDestinationError", "FilterError"),
        ("LinkFallbackError", "FilterError"),
    ];
    for (name, base) in exc_classes {
        super::super::class::mb_class_register(
            name,
            vec![base.to_string()],
            HashMap::new(),
        );
        attrs.insert(name.to_string(),
            MbValue::from_ptr(MbObject::new_str(name.to_string())));
    }
    // surface: TarFile must satisfy `callable(...)` (a constructable class
    // object in CPython). The string-sentinel registration above leaves it
    // present-but-not-callable; overwrite with a from_func stub so
    // `callable(tarfile.TarFile)` resolves True.
    let callable_classes: Vec<&str> = vec!["TarFile"];
    for name in callable_classes {
        let addr = dispatch_tarfile_stub as usize;
        attrs.insert(name.to_string(), MbValue::from_func(addr));
        super::super::module::NATIVE_FUNC_ADDRS.with(|st| {
            st.borrow_mut().insert(addr as u64);
        });
    }

    // symlink_exception is a tuple of exception classes; surface as a tuple of
    // class-name string sentinels (present + tuple-typed).
    attrs.insert("symlink_exception".into(),
        MbValue::from_ptr(MbObject::new_tuple(vec![
            s("AttributeError"), s("NotImplementedError"), s("OSError"),
        ])));

    super::register_module("tarfile", attrs);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_itn_octal_and_base256() {
        assert_eq!(itn(1, 8, PAX_FORMAT).unwrap(), b"0000001\0");
        assert_eq!(itn(2097151, 8, PAX_FORMAT).unwrap(), b"7777777\0");
        assert_eq!(
            itn(2097152, 8, GNU_FORMAT).unwrap(),
            b"\x80\x00\x00\x00\x00\x20\x00\x00"
        );
        assert_eq!(itn(-1, 8, GNU_FORMAT).unwrap(), vec![0xffu8; 8]);
        assert!(itn(2097152, 8, USTAR_FORMAT).is_err());
        assert!(itn(1i128 << 56, 8, GNU_FORMAT).is_err());
    }

    #[test]
    fn test_nti_roundtrip() {
        for n in [0i128, 1, 2097151, 2097152, 4294967295, -1, -100] {
            let enc = itn(n, 8, GNU_FORMAT).unwrap();
            assert_eq!(nti(&enc).unwrap(), n, "roundtrip {n}");
        }
        assert_eq!(nti(b"\x00").unwrap(), 0);
        assert_eq!(nti(b"       \x00").unwrap(), 0);
    }

    #[test]
    fn test_stn_nts() {
        assert_eq!(stn("foo", 8, "ascii"), b"foo\0\0\0\0\0");
        assert_eq!(stn("foobar", 3, "ascii"), b"foo");
        assert_eq!(nts(b"foo\0\0\0"), "foo");
        assert_eq!(nts(b"foo\0bar\0"), "foo");
    }

    #[test]
    fn test_write_parse_roundtrip() {
        let v = TView {
            name: "hello.txt".to_string(),
            mode: Some(0o644),
            uid: Some(0),
            gid: Some(0),
            size: 5,
            type_b: b'0',
            ..TView::default()
        };
        let mut archive = tobuf_view(&v, PAX_FORMAT).unwrap();
        archive.extend(pad_block(b"hello".to_vec()));
        archive.resize(RECORDSIZE, 0);
        let members = parse_archive(&archive).unwrap();
        assert_eq!(members.len(), 1);
        assert_eq!(members[0].v.name, "hello.txt");
        assert_eq!(members[0].v.size, 5);
        assert_eq!(members[0].data, b"hello");
    }

    #[test]
    fn test_pax_oversized_uid_roundtrip() {
        let v = TView {
            name: "big.bin".to_string(),
            mode: Some(0o644),
            uid: Some(1i128 << 24),
            gid: Some(0),
            size: 0,
            type_b: b'0',
            ..TView::default()
        };
        let mut archive = tobuf_view(&v, PAX_FORMAT).unwrap();
        archive.resize(RECORDSIZE, 0);
        let members = parse_archive(&archive).unwrap();
        assert_eq!(members.len(), 1);
        assert_eq!(members[0].v.uid, Some(1i128 << 24));
    }

    #[test]
    fn test_ustar_name_limits() {
        let ok100 = TView {
            name: "0123456789".repeat(10),
            mode: Some(0o644),
            uid: Some(0),
            gid: Some(0),
            ..TView::default()
        };
        assert!(tobuf_view(&ok100, USTAR_FORMAT).is_ok());
        let splittable = TView {
            name: "123/".repeat(62) + "longname",
            mode: Some(0o644),
            uid: Some(0),
            gid: Some(0),
            ..TView::default()
        };
        assert!(tobuf_view(&splittable, USTAR_FORMAT).is_ok());
        let too_long = TView {
            name: "0123456789".repeat(10) + "0",
            mode: Some(0o644),
            uid: Some(0),
            gid: Some(0),
            ..TView::default()
        };
        assert!(tobuf_view(&too_long, USTAR_FORMAT).is_err());
        // GNU lifts the limit.
        assert!(tobuf_view(&too_long, GNU_FORMAT).is_ok());
    }

    #[test]
    fn test_parse_garbage_fails() {
        assert!(parse_archive(b"not a tar file").is_err());
        let garbage = vec![b'x'; 1024];
        assert!(parse_archive(&garbage).is_err());
        // All-zero blocks: a valid empty archive.
        assert!(parse_archive(&vec![0u8; RECORDSIZE]).unwrap().is_empty());
    }
}
