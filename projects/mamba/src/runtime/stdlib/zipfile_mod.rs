use super::super::rc::{MbObject, MbObjectHeader, ObjData, ObjKind};
use super::super::value::MbValue;
use crate::runtime::rc::MbRwLock as RwLock;
use rustc_hash::FxHashMap;
/// zipfile module for Mamba (#445) — real ZIP container engine.
///
/// ZipFile reads and writes the actual ZIP format (local file headers,
/// central directory, EOCD with archive comment) against a filesystem path
/// or an io.BytesIO-like object, with ZIP_STORED and ZIP_DEFLATED (flate2)
/// members, CRC-32 validation (BadZipFile on mismatch, testzip reporting),
/// append mode, extract/extractall to disk, member streaming via open()
/// (read and write sides, including force_zip64 local-header layout), and
/// ZipInfo records (defaults, from_file, is_dir, repr, pre-1980 ValueError).
use std::collections::HashMap;
use std::io::{Read, Write};
use std::sync::atomic::AtomicU32;

const ZIPFILE_CLASS: &str = "ZipFile";
const ZIPINFO_CLASS: &str = "ZipInfo";

// ── Instance-field + value helpers ──

fn set_field(inst: MbValue, key: &str, val: MbValue) {
    if let Some(ptr) = inst.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                super::super::rc::retain_if_ptr(val);
                let prev = fields.write().unwrap().insert(key.to_string(), val);
                if let Some(p) = prev {
                    super::super::rc::release_if_ptr(p);
                }
            }
        }
    }
}

fn get_field(inst: MbValue, key: &str) -> Option<MbValue> {
    inst.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Instance { ref fields, .. } = (*ptr).data {
            fields.read().unwrap().get(key).copied()
        } else {
            None
        }
    })
}

fn make_instance(class_name: &str, fields_kv: Vec<(&str, MbValue)>) -> MbValue {
    let mut fields = FxHashMap::default();
    for (k, v) in fields_kv {
        fields.insert(k.to_string(), v);
    }
    let obj = Box::new(MbObject {
        header: MbObjectHeader {
            rc: AtomicU32::new(1),
            kind: ObjKind::Instance,
        },
        data: ObjData::Instance {
            class_name: class_name.to_string(),
            fields: RwLock::new(fields),
        },
    });
    MbValue::from_ptr(Box::into_raw(obj))
}

fn new_str(s: &str) -> MbValue {
    MbValue::from_ptr(MbObject::new_str(s.to_string()))
}

fn new_bytes(b: Vec<u8>) -> MbValue {
    MbValue::from_ptr(MbObject::new_bytes(b))
}

fn extract_str(val: MbValue) -> Option<String> {
    val.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Str(ref s) = (*ptr).data {
            Some(s.clone())
        } else {
            None
        }
    })
}

fn extract_bytes(val: MbValue) -> Option<Vec<u8>> {
    val.as_ptr().and_then(|ptr| unsafe {
        match &(*ptr).data {
            ObjData::Bytes(b) => Some(b.clone()),
            ObjData::ByteArray(lock) => Some(lock.read().unwrap().clone()),
            _ => None,
        }
    })
}

fn seq_items(val: MbValue) -> Vec<MbValue> {
    val.as_ptr()
        .and_then(|ptr| unsafe {
            match &(*ptr).data {
                ObjData::List(lock) => lock.read().ok().map(|g| g.to_vec()),
                ObjData::Tuple(items) => Some(items.clone()),
                _ => None,
            }
        })
        .unwrap_or_default()
}

fn is_dict_value(v: MbValue) -> bool {
    v.as_ptr()
        .map(|p| unsafe { matches!((*p).data, ObjData::Dict(_)) })
        .unwrap_or(false)
}

fn kwarg(kw: MbValue, name: &str) -> Option<MbValue> {
    if kw.is_none() {
        return None;
    }
    let sentinel = MbValue::from_bits(u64::MAX);
    let v = super::super::dict_ops::mb_dict_get(kw, new_str(name), sentinel);
    if v.to_bits() == u64::MAX {
        None
    } else {
        Some(v)
    }
}

fn raise_str(exc: &str, msg: &str) -> MbValue {
    super::super::exception::mb_raise(new_str(exc), new_str(msg));
    MbValue::none()
}

unsafe fn arg_slice<'a>(args_ptr: *const MbValue, nargs: usize) -> &'a [MbValue] {
    if nargs == 0 || args_ptr.is_null() {
        &[]
    } else {
        unsafe { std::slice::from_raw_parts(args_ptr, nargs) }
    }
}

// ── ZIP byte-level helpers ──

fn crc32(data: &[u8]) -> u32 {
    let mut crc = flate2::Crc::new();
    crc.update(data);
    crc.sum()
}

fn deflate(data: &[u8]) -> Vec<u8> {
    let mut enc = flate2::write::DeflateEncoder::new(Vec::new(), flate2::Compression::default());
    let _ = enc.write_all(data);
    enc.finish().unwrap_or_default()
}

fn inflate(data: &[u8]) -> Vec<u8> {
    let mut dec = flate2::read::DeflateDecoder::new(data);
    let mut out = Vec::new();
    let _ = dec.read_to_end(&mut out);
    out
}

fn u16le(b: &[u8], off: usize) -> u32 {
    if off + 2 > b.len() {
        return 0;
    }
    u16::from_le_bytes([b[off], b[off + 1]]) as u32
}

fn u32le(b: &[u8], off: usize) -> u64 {
    if off + 4 > b.len() {
        return 0;
    }
    u32::from_le_bytes([b[off], b[off + 1], b[off + 2], b[off + 3]]) as u64
}

fn u64le(b: &[u8], off: usize) -> u64 {
    if off + 8 > b.len() {
        return 0;
    }
    u64::from_le_bytes(b[off..off + 8].try_into().unwrap())
}

fn push_u16(out: &mut Vec<u8>, v: u32) {
    out.extend_from_slice(&(v as u16).to_le_bytes());
}

fn push_u32(out: &mut Vec<u8>, v: u64) {
    out.extend_from_slice(&(v as u32).to_le_bytes());
}

/// DOS (time, date) words from a (Y, M, D, h, m, s) tuple.
fn dos_datetime(dt: &[i64]) -> (u32, u32) {
    let (y, mo, d, h, mi, s) = (
        dt.first().copied().unwrap_or(1980),
        dt.get(1).copied().unwrap_or(1),
        dt.get(2).copied().unwrap_or(1),
        dt.get(3).copied().unwrap_or(0),
        dt.get(4).copied().unwrap_or(0),
        dt.get(5).copied().unwrap_or(0),
    );
    let time = ((h as u32) << 11) | ((mi as u32) << 5) | ((s as u32) / 2);
    let date = (((y - 1980).max(0) as u32) << 9) | ((mo as u32) << 5) | (d as u32);
    (time, date)
}

fn dt_tuple_from_dos(time: u32, date: u32) -> Vec<i64> {
    vec![
        (date >> 9) as i64 + 1980,
        ((date >> 5) & 0xF) as i64,
        (date & 0x1F) as i64,
        (time >> 11) as i64,
        ((time >> 5) & 0x3F) as i64,
        ((time & 0x1F) * 2) as i64,
    ]
}

// ── ZipInfo ──

fn date_time_value(dt: &[i64]) -> MbValue {
    let items: Vec<MbValue> = dt.iter().map(|v| MbValue::from_int(*v)).collect();
    MbValue::from_ptr(MbObject::new_tuple(items))
}

/// Build a ZipInfo instance with CPython's default attribute set.
fn make_zipinfo(filename: &str, date_time: &[i64]) -> MbValue {
    make_instance(
        ZIPINFO_CLASS,
        vec![
            ("orig_filename", new_str(filename)),
            ("filename", new_str(filename)),
            ("date_time", date_time_value(date_time)),
            ("compress_type", MbValue::from_int(0)),
            ("comment", new_bytes(Vec::new())),
            ("extra", new_bytes(Vec::new())),
            ("create_system", MbValue::from_int(3)),
            ("create_version", MbValue::from_int(20)),
            ("extract_version", MbValue::from_int(20)),
            ("reserved", MbValue::from_int(0)),
            ("flag_bits", MbValue::from_int(0)),
            ("volume", MbValue::from_int(0)),
            ("internal_attr", MbValue::from_int(0)),
            ("external_attr", MbValue::from_int(0)),
            ("file_size", MbValue::from_int(0)),
            ("compress_size", MbValue::from_int(0)),
            ("CRC", MbValue::from_int(0)),
            ("header_offset", MbValue::from_int(0)),
            ("_compress_level", MbValue::none()),
        ],
    )
}

fn zi_int(zi: MbValue, key: &str) -> i64 {
    get_field(zi, key).and_then(|v| v.as_int()).unwrap_or(0)
}

fn zi_str(zi: MbValue, key: &str) -> String {
    get_field(zi, key).and_then(extract_str).unwrap_or_default()
}

fn zi_datetime(zi: MbValue) -> Vec<i64> {
    get_field(zi, "date_time")
        .map(seq_items)
        .unwrap_or_default()
        .iter()
        .map(|v| v.as_int().unwrap_or(0))
        .collect()
}

unsafe extern "C" fn d_zipinfo_new(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { arg_slice(args_ptr, nargs) };
    let kw = a
        .iter()
        .copied()
        .find(|v| is_dict_value(*v))
        .unwrap_or_else(MbValue::none);
    let pos: Vec<MbValue> = a.iter().copied().filter(|v| !is_dict_value(*v)).collect();
    let filename = kwarg(kw, "filename")
        .or_else(|| pos.first().copied())
        .and_then(extract_str)
        .unwrap_or_else(|| "NoName".to_string());
    // NUL truncates the name (defends against appended garbage).
    let filename = filename.split('\0').next().unwrap_or("").to_string();
    let dt: Vec<i64> = kwarg(kw, "date_time")
        .or_else(|| pos.get(1).copied())
        .map(seq_items)
        .map(|items| items.iter().map(|v| v.as_int().unwrap_or(0)).collect())
        .unwrap_or_else(|| vec![1980, 1, 1, 0, 0, 0]);
    if dt.first().copied().unwrap_or(1980) < 1980 {
        return raise_str("ValueError", "ZIP does not support timestamps before 1980");
    }
    make_zipinfo(&filename, &dt)
}

/// `ZipInfo.from_file(path, arcname=None)` classmethod.
unsafe extern "C" fn d_zipinfo_from_file(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { arg_slice(args_ptr, nargs) };
    let Some(path) = a.first().copied().and_then(extract_str) else {
        return raise_str("TypeError", "from_file() requires a path");
    };
    let arcname = a
        .get(1)
        .copied()
        .filter(|v| !is_dict_value(*v))
        .and_then(extract_str);
    let meta = match std::fs::metadata(&path) {
        Ok(m) => m,
        Err(e) => return raise_str("OSError", &format!("{e}: {path:?}")),
    };
    let is_dir = meta.is_dir();
    let mut name = arcname.unwrap_or_else(|| path.clone());
    name = name.trim_start_matches('/').to_string();
    if is_dir && !name.ends_with('/') {
        name.push('/');
    }
    let zi = make_zipinfo(&name, &[1980, 1, 1, 0, 0, 0]);
    if !is_dir {
        set_field(zi, "file_size", MbValue::from_int(meta.len() as i64));
        set_field(zi, "external_attr", MbValue::from_int(0o600 << 16));
    } else {
        set_field(
            zi,
            "external_attr",
            MbValue::from_int((0o40775 << 16) | 0x10),
        );
    }
    zi
}

unsafe extern "C" fn zipinfo_is_dir(self_v: MbValue, _args: MbValue) -> MbValue {
    MbValue::from_bool(zi_str(self_v, "filename").ends_with('/'))
}

unsafe extern "C" fn zipinfo_repr(self_v: MbValue, _args: MbValue) -> MbValue {
    let name = zi_str(self_v, "filename");
    let comp = zi_int(self_v, "compress_type");
    let size = zi_int(self_v, "file_size");
    if comp != 0 {
        new_str(&format!(
            "<ZipInfo filename='{name}' compress_type={comp} file_size={size}>"
        ))
    } else {
        new_str(&format!("<ZipInfo filename='{name}' file_size={size}>"))
    }
}

// ── ZIP container parse / serialize ──

struct Entry {
    info: MbValue,
    cdata: Vec<u8>,
    force_zip64: bool,
}

/// Parse a ZIP byte blob into (entries, archive comment). Err = not a zip.
fn parse_zip(buf: &[u8]) -> Result<(Vec<Entry>, Vec<u8>), ()> {
    // Locate EOCD: scan back for PK\x05\x06.
    let mut eocd = None;
    if buf.len() >= 22 {
        let start = buf.len().saturating_sub(22 + 65536);
        let mut i = buf.len() - 22;
        loop {
            if &buf[i..i + 4] == b"PK\x05\x06" {
                eocd = Some(i);
                break;
            }
            if i == start {
                break;
            }
            i -= 1;
        }
    }
    let Some(e) = eocd else { return Err(()) };
    let count = u16le(buf, e + 10) as usize;
    let cd_off = u32le(buf, e + 16) as usize;
    let comment_len = u16le(buf, e + 20) as usize;
    let comment = buf
        .get(e + 22..e + 22 + comment_len)
        .unwrap_or(&[])
        .to_vec();

    let mut entries = Vec::new();
    let mut p = cd_off;
    for _ in 0..count {
        if p + 46 > buf.len() || &buf[p..p + 4] != b"PK\x01\x02" {
            return Err(());
        }
        let extract_version = u16le(buf, p + 6) & 0xFF;
        let flags = u16le(buf, p + 8);
        let comp = u16le(buf, p + 10);
        let dtime = u16le(buf, p + 12);
        let ddate = u16le(buf, p + 14);
        let crc = u32le(buf, p + 16);
        let mut csize = u32le(buf, p + 20);
        let mut usize_ = u32le(buf, p + 24);
        let fn_len = u16le(buf, p + 28) as usize;
        let extra_len = u16le(buf, p + 30) as usize;
        let cmt_len = u16le(buf, p + 32) as usize;
        let external_attr = u32le(buf, p + 38);
        let mut header_off = u32le(buf, p + 42) as usize;
        let name_raw = buf.get(p + 46..p + 46 + fn_len).unwrap_or(&[]);
        let name = if flags & 0x800 != 0 {
            String::from_utf8_lossy(name_raw).to_string()
        } else {
            String::from_utf8_lossy(name_raw).to_string()
        };
        // ZIP64 extra (id 0x0001) overrides 0xFFFFFFFF sentinels.
        let extra = buf
            .get(p + 46 + fn_len..p + 46 + fn_len + extra_len)
            .unwrap_or(&[]);
        let mut q = 0usize;
        while q + 4 <= extra.len() {
            let id = u16le(extra, q);
            let len = u16le(extra, q + 2) as usize;
            if id == 1 {
                let mut r = q + 4;
                if usize_ == 0xFFFF_FFFF && r + 8 <= extra.len() {
                    usize_ = u64le(extra, r);
                    r += 8;
                }
                if csize == 0xFFFF_FFFF && r + 8 <= extra.len() {
                    csize = u64le(extra, r);
                    r += 8;
                }
                if header_off == 0xFFFF_FFFF as usize && r + 8 <= extra.len() {
                    header_off = u64le(extra, r) as usize;
                }
            }
            q += 4 + len;
        }
        // Local header: skip 30 + its own name/extra lengths to the data.
        if header_off + 30 > buf.len() || &buf[header_off..header_off + 4] != b"PK\x03\x04" {
            return Err(());
        }
        let lfn = u16le(buf, header_off + 26) as usize;
        let lex = u16le(buf, header_off + 28) as usize;
        let data_off = header_off + 30 + lfn + lex;
        let cdata = buf
            .get(data_off..data_off + csize as usize)
            .unwrap_or(&[])
            .to_vec();

        let zi = make_zipinfo(&name, &dt_tuple_from_dos(dtime, ddate));
        set_field(zi, "compress_type", MbValue::from_int(comp as i64));
        set_field(zi, "CRC", MbValue::from_int(crc as i64));
        set_field(zi, "compress_size", MbValue::from_int(csize as i64));
        set_field(zi, "file_size", MbValue::from_int(usize_ as i64));
        set_field(zi, "header_offset", MbValue::from_int(header_off as i64));
        set_field(zi, "external_attr", MbValue::from_int(external_attr as i64));
        set_field(
            zi,
            "extract_version",
            MbValue::from_int(extract_version as i64),
        );
        set_field(zi, "flag_bits", MbValue::from_int(flags as i64));
        entries.push(Entry {
            info: zi,
            cdata,
            force_zip64: false,
        });

        p += 46 + fn_len + extra_len + cmt_len;
    }
    Ok((entries, comment))
}

/// Serialize entries + comment into the ZIP byte format.
fn serialize_zip(entries: &[Entry], comment: &[u8]) -> Vec<u8> {
    let mut out: Vec<u8> = Vec::new();
    let mut centrals: Vec<(usize, Vec<u8>)> = Vec::new();
    for e in entries {
        let name = zi_str(e.info, "filename");
        let name_b = name.as_bytes();
        let utf8_flag = if name.is_ascii() { 0 } else { 0x800u32 };
        let comp = zi_int(e.info, "compress_type") as u32;
        let crc = zi_int(e.info, "CRC") as u64;
        let csize = e.cdata.len() as u64;
        let usize_ = zi_int(e.info, "file_size") as u64;
        let (dtime, ddate) = dos_datetime(&zi_datetime(e.info));
        let need64 = e.force_zip64 || csize >= 0xFFFF_FFFF || usize_ >= 0xFFFF_FFFF;
        let version = if need64 { 45 } else { 20 };
        let header_off = out.len();
        set_field(
            e.info,
            "header_offset",
            MbValue::from_int(header_off as i64),
        );

        // Local file header.
        out.extend_from_slice(b"PK\x03\x04");
        push_u16(&mut out, version);
        push_u16(&mut out, utf8_flag);
        push_u16(&mut out, comp);
        push_u16(&mut out, dtime);
        push_u16(&mut out, ddate);
        push_u32(&mut out, crc);
        if need64 {
            push_u32(&mut out, 0xFFFF_FFFF);
            push_u32(&mut out, 0xFFFF_FFFF);
        } else {
            push_u32(&mut out, csize);
            push_u32(&mut out, usize_);
        }
        push_u16(&mut out, name_b.len() as u32);
        let extra_len = if need64 { 20u32 } else { 0 };
        push_u16(&mut out, extra_len);
        out.extend_from_slice(name_b);
        if need64 {
            push_u16(&mut out, 1);
            push_u16(&mut out, 16);
            out.extend_from_slice(&usize_.to_le_bytes());
            out.extend_from_slice(&csize.to_le_bytes());
        }
        out.extend_from_slice(&e.cdata);

        // Central directory record.
        let mut c: Vec<u8> = Vec::new();
        c.extend_from_slice(b"PK\x01\x02");
        push_u16(&mut c, version | (3 << 8)); // made by: unix
        push_u16(&mut c, version);
        push_u16(&mut c, utf8_flag);
        push_u16(&mut c, comp);
        push_u16(&mut c, dtime);
        push_u16(&mut c, ddate);
        push_u32(&mut c, crc);
        if need64 {
            push_u32(&mut c, 0xFFFF_FFFF);
            push_u32(&mut c, 0xFFFF_FFFF);
        } else {
            push_u32(&mut c, csize);
            push_u32(&mut c, usize_);
        }
        push_u16(&mut c, name_b.len() as u32);
        push_u16(&mut c, if need64 { 20 } else { 0 });
        push_u16(&mut c, 0); // comment len
        push_u16(&mut c, 0); // disk
        push_u16(&mut c, 0); // internal attrs
        push_u32(&mut c, zi_int(e.info, "external_attr") as u64);
        push_u32(&mut c, header_off as u64);
        c.extend_from_slice(name_b);
        if need64 {
            push_u16(&mut c, 1);
            push_u16(&mut c, 16);
            c.extend_from_slice(&usize_.to_le_bytes());
            c.extend_from_slice(&csize.to_le_bytes());
        }
        centrals.push((header_off, c));
    }
    let cd_start = out.len();
    for (_, c) in &centrals {
        out.extend_from_slice(c);
    }
    let cd_size = out.len() - cd_start;
    out.extend_from_slice(b"PK\x05\x06");
    push_u16(&mut out, 0);
    push_u16(&mut out, 0);
    push_u16(&mut out, entries.len() as u32);
    push_u16(&mut out, entries.len() as u32);
    push_u32(&mut out, cd_size as u64);
    push_u32(&mut out, cd_start as u64);
    push_u16(&mut out, comment.len() as u32);
    out.extend_from_slice(comment);
    out
}

// ── ZipFile target I/O (path or BytesIO-like) ──

fn target_read(target: MbValue) -> Option<Vec<u8>> {
    if let Some(path) = extract_str(target) {
        return std::fs::read(path).ok();
    }
    // BytesIO-like: `_buffer` field.
    get_field(target, "_buffer").and_then(extract_bytes)
}

fn target_write(target: MbValue, data: Vec<u8>) {
    if let Some(path) = extract_str(target) {
        let _ = std::fs::write(path, data);
        return;
    }
    if get_field(target, "_buffer").is_some() {
        set_field(target, "_buffer", new_bytes(data));
    }
}

// ── per-handle entry store ──

struct ZfState {
    entries: Vec<Entry>,
    comment: Vec<u8>,
    mode: char,
    closed: bool,
}

thread_local! {
    static ZF_STORES: std::cell::RefCell<FxHashMap<u64, ZfState>> =
        std::cell::RefCell::new(FxHashMap::default());
    static ZF_NEXT_ID: std::cell::Cell<u64> = const { std::cell::Cell::new(1) };
}

fn zf_id(zf: MbValue) -> Option<u64> {
    get_field(zf, "_id")
        .and_then(|v| v.as_int())
        .map(|i| i as u64)
}

fn with_zf<R>(zf: MbValue, f: impl FnOnce(&mut ZfState) -> R) -> Option<R> {
    let id = zf_id(zf)?;
    ZF_STORES.with(|stores| stores.borrow_mut().get_mut(&id).map(f))
}

/// Refresh the `filelist` field from the store so attribute reads see the
/// live entry list.
fn refresh_filelist(zf: MbValue) {
    let infos: Vec<MbValue> =
        with_zf(zf, |st| st.entries.iter().map(|e| e.info).collect()).unwrap_or_default();
    set_field(zf, "filelist", MbValue::from_ptr(MbObject::new_list(infos)));
}

// ── ZipFile constructor ──

unsafe extern "C" fn d_zipfile_new(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { arg_slice(args_ptr, nargs) };
    let kw = a
        .iter()
        .copied()
        .rev()
        .find(|v| is_dict_value(*v))
        .unwrap_or_else(MbValue::none);
    let pos: Vec<MbValue> = a.iter().copied().filter(|v| !is_dict_value(*v)).collect();
    let target = pos.first().copied().unwrap_or_else(MbValue::none);
    let mode = kwarg(kw, "mode")
        .or_else(|| pos.get(1).copied())
        .and_then(extract_str)
        .unwrap_or_else(|| "r".to_string())
        .chars()
        .next()
        .unwrap_or('r');
    let compression = kwarg(kw, "compression")
        .or_else(|| pos.get(2).copied().filter(|v| v.as_int().is_some()))
        .and_then(|v| v.as_int())
        .unwrap_or(0);
    if !matches!(mode, 'r' | 'w' | 'x' | 'a') {
        return raise_str("ValueError", "ZipFile requires mode 'r', 'w', 'x', or 'a'");
    }
    // STORED / DEFLATED / BZIP2 / LZMA are the known methods.
    if !matches!(compression, 0 | 8 | 12 | 14) {
        return raise_str(
            "NotImplementedError",
            "That compression method is not supported",
        );
    }

    let mut entries = Vec::new();
    let mut comment = Vec::new();
    if mode == 'r' || mode == 'a' {
        let existing = target_read(target);
        match existing {
            Some(buf) if !buf.is_empty() => match parse_zip(&buf) {
                Ok((e, c)) => {
                    entries = e;
                    comment = c;
                }
                Err(()) => {
                    if mode == 'r' {
                        return raise_str("zipfile.BadZipFile", "File is not a zip file");
                    }
                }
            },
            Some(_) => {
                // The target exists but is empty — not a zip archive.
                if mode == 'r' {
                    return raise_str("zipfile.BadZipFile", "File is not a zip file");
                }
            }
            None => {
                if mode == 'r' {
                    return raise_str(
                        "OSError",
                        &format!(
                            "[Errno 2] No such file or directory: {:?}",
                            extract_str(target).unwrap_or_default()
                        ),
                    );
                }
            }
        }
    }

    let id = ZF_NEXT_ID.with(|c| {
        let v = c.get();
        c.set(v + 1);
        v
    });
    let comment_v = new_bytes(comment.clone());
    ZF_STORES.with(|stores| {
        stores.borrow_mut().insert(
            id,
            ZfState {
                entries,
                comment,
                mode,
                closed: false,
            },
        );
    });
    let zf = make_instance(
        ZIPFILE_CLASS,
        vec![
            ("_id", MbValue::from_int(id as i64)),
            ("_target", target),
            ("mode", new_str(&mode.to_string())),
            ("compression", MbValue::from_int(compression)),
            ("comment", comment_v),
            ("fp", target),
        ],
    );
    refresh_filelist(zf);
    zf
}

// ── write paths ──

fn compress_payload(data: &[u8], comp: i64) -> Vec<u8> {
    if comp == 8 {
        deflate(data)
    } else {
        data.to_vec()
    }
}

/// ValueError when the archive handle is already closed (CPython contract).
fn ensure_zf_open(zf: MbValue) -> Option<MbValue> {
    let closed = with_zf(zf, |st| st.closed).unwrap_or(true);
    if closed {
        return Some(raise_str(
            "ValueError",
            "Attempt to use ZIP archive that was already closed",
        ));
    }
    None
}

/// Core writestr: name_or_info + raw data (+ force_zip64 from open('w')).
fn writestr_core(zf: MbValue, name_or_info: MbValue, data: Vec<u8>, force_zip64: bool) -> MbValue {
    let closed = with_zf(zf, |st| st.closed).unwrap_or(true);
    if closed {
        return raise_str(
            "ValueError",
            "Attempt to use ZIP archive that was already closed",
        );
    }
    let default_comp = get_field(zf, "compression")
        .and_then(|v| v.as_int())
        .unwrap_or(0);
    let (info, comp) = if extract_str(name_or_info).is_some() {
        let raw_name = extract_str(name_or_info).unwrap_or_default();
        let name = raw_name.split('\0').next().unwrap_or("").to_string();
        let zi = make_zipinfo(&name, &[1980, 1, 1, 0, 0, 0]);
        set_field(zi, "compress_type", MbValue::from_int(default_comp));
        (zi, default_comp)
    } else {
        let comp = zi_int(name_or_info, "compress_type");
        (name_or_info, comp)
    };
    let crc = crc32(&data);
    let cdata = compress_payload(&data, comp);
    set_field(info, "file_size", MbValue::from_int(data.len() as i64));
    set_field(info, "compress_size", MbValue::from_int(cdata.len() as i64));
    set_field(info, "CRC", MbValue::from_int(crc as i64));
    if force_zip64 {
        set_field(info, "extract_version", MbValue::from_int(45));
        set_field(info, "create_version", MbValue::from_int(45));
    }
    with_zf(zf, |st| {
        st.entries.push(Entry {
            info,
            cdata,
            force_zip64,
        });
    });
    refresh_filelist(zf);
    MbValue::none()
}

unsafe extern "C" fn method_writestr(self_v: MbValue, args: MbValue) -> MbValue {
    let items = seq_items(args);
    let name_or_info = items.first().copied().unwrap_or_else(MbValue::none);
    let raw = items.get(1).copied().unwrap_or_else(MbValue::none);
    let data = extract_bytes(raw)
        .or_else(|| extract_str(raw).map(|s| s.into_bytes()))
        .unwrap_or_default();
    writestr_core(self_v, name_or_info, data, false)
}

/// `zf.write(path, arcname=None)` — add a real file from disk.
unsafe extern "C" fn method_write(self_v: MbValue, args: MbValue) -> MbValue {
    let items = seq_items(args);
    let Some(path) = items.first().copied().and_then(extract_str) else {
        return raise_str("TypeError", "write() requires a path");
    };
    let arcname = items
        .get(1)
        .copied()
        .and_then(extract_str)
        .unwrap_or_else(|| path.clone());
    match std::fs::read(&path) {
        Ok(data) => writestr_core(self_v, new_str(&arcname), data, false),
        Err(e) => raise_str("OSError", &format!("{e}: {path:?}")),
    }
}

// ── read paths ──

fn find_entry_payload(zf: MbValue, name: &str) -> Option<(MbValue, Vec<u8>)> {
    with_zf(zf, |st| {
        st.entries
            .iter()
            .find(|e| zi_str(e.info, "filename") == name)
            .map(|e| (e.info, e.cdata.clone()))
    })
    .flatten()
}

fn decompress_entry(info: MbValue, cdata: &[u8]) -> Vec<u8> {
    if zi_int(info, "compress_type") == 8 {
        inflate(cdata)
    } else {
        cdata.to_vec()
    }
}

// ── shutil bridge: whole-archive pack/unpack over (name, data) pairs ──

/// Build a ZIP byte blob (ZIP_STORED) from (arcname, contents) pairs.
/// Used by shutil.make_archive.
pub(crate) fn zip_pack(files: &[(String, Vec<u8>)]) -> Vec<u8> {
    let mut entries: Vec<Entry> = Vec::with_capacity(files.len());
    for (name, data) in files {
        let zi = make_zipinfo(name, &[1980, 1, 1, 0, 0, 0]);
        set_field(zi, "compress_type", MbValue::from_int(0));
        set_field(zi, "file_size", MbValue::from_int(data.len() as i64));
        set_field(zi, "compress_size", MbValue::from_int(data.len() as i64));
        set_field(zi, "CRC", MbValue::from_int(crc32(data) as i64));
        entries.push(Entry {
            info: zi,
            cdata: data.clone(),
            force_zip64: false,
        });
    }
    serialize_zip(&entries, &[])
}

/// Parse a ZIP byte blob into (arcname, contents) pairs (decompressed).
/// Used by shutil.unpack_archive. None when the blob is not a zip.
pub(crate) fn zip_unpack(buf: &[u8]) -> Option<Vec<(String, Vec<u8>)>> {
    let (entries, _comment) = parse_zip(buf).ok()?;
    Some(
        entries
            .iter()
            .map(|e| {
                (
                    zi_str(e.info, "filename"),
                    decompress_entry(e.info, &e.cdata),
                )
            })
            .collect(),
    )
}

unsafe extern "C" fn method_read(self_v: MbValue, args: MbValue) -> MbValue {
    if let Some(err) = ensure_zf_open(self_v) {
        return err;
    }
    let items = seq_items(args);
    let name = items
        .first()
        .copied()
        .and_then(extract_str)
        .unwrap_or_default();
    let Some((info, cdata)) = find_entry_payload(self_v, &name) else {
        return raise_str(
            "KeyError",
            &format!("There is no item named {name:?} in the archive"),
        );
    };
    let data = decompress_entry(info, &cdata);
    if crc32(&data) as i64 != zi_int(info, "CRC") {
        return raise_str(
            "zipfile.BadZipFile",
            &format!("Bad CRC-32 for file {name:?}"),
        );
    }
    new_bytes(data)
}

unsafe extern "C" fn method_namelist(self_v: MbValue, _args: MbValue) -> MbValue {
    let names: Vec<MbValue> = with_zf(self_v, |st| {
        st.entries
            .iter()
            .map(|e| new_str(&zi_str(e.info, "filename")))
            .collect()
    })
    .unwrap_or_default();
    MbValue::from_ptr(MbObject::new_list(names))
}

unsafe extern "C" fn method_infolist(self_v: MbValue, _args: MbValue) -> MbValue {
    let infos: Vec<MbValue> =
        with_zf(self_v, |st| st.entries.iter().map(|e| e.info).collect()).unwrap_or_default();
    MbValue::from_ptr(MbObject::new_list(infos))
}

unsafe extern "C" fn method_getinfo(self_v: MbValue, args: MbValue) -> MbValue {
    let items = seq_items(args);
    let name = items
        .first()
        .copied()
        .and_then(extract_str)
        .unwrap_or_default();
    match find_entry_payload(self_v, &name) {
        Some((info, _)) => info,
        None => raise_str(
            "KeyError",
            &format!("There is no item named {name:?} in the archive"),
        ),
    }
}

unsafe extern "C" fn method_testzip(self_v: MbValue, _args: MbValue) -> MbValue {
    if let Some(err) = ensure_zf_open(self_v) {
        return err;
    }
    let bad: Option<String> = with_zf(self_v, |st| {
        for e in &st.entries {
            let data = decompress_entry(e.info, &e.cdata);
            if crc32(&data) as i64 != zi_int(e.info, "CRC") {
                return Some(zi_str(e.info, "filename"));
            }
        }
        None
    })
    .flatten();
    match bad {
        Some(name) => new_str(&name),
        None => MbValue::none(),
    }
}

// ── open() — read and write member streams ──

unsafe extern "C" fn method_open(self_v: MbValue, args: MbValue) -> MbValue {
    if let Some(err) = ensure_zf_open(self_v) {
        return err;
    }
    let items = seq_items(args);
    let kw = items
        .iter()
        .copied()
        .find(|v| is_dict_value(*v))
        .unwrap_or_else(MbValue::none);
    let name = items
        .first()
        .copied()
        .and_then(extract_str)
        .unwrap_or_default();
    let mode = kwarg(kw, "mode")
        .or_else(|| items.get(1).copied().filter(|v| !is_dict_value(*v)))
        .and_then(extract_str)
        .unwrap_or_else(|| "r".to_string());
    if mode != "r" && mode != "w" {
        return raise_str("ValueError", "open() requires mode \"r\" or \"w\"");
    }
    if mode.starts_with('w') {
        let force_zip64 = kwarg(kw, "force_zip64")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        return make_instance(
            "zipfile._ZipWriteFile",
            vec![
                ("_zf", self_v),
                ("_name", new_str(&name)),
                ("_force64", MbValue::from_bool(force_zip64)),
                ("_chunks", new_bytes(Vec::new())),
            ],
        );
    }
    let Some((info, cdata)) = find_entry_payload(self_v, &name) else {
        return raise_str(
            "KeyError",
            &format!("There is no item named {name:?} in the archive"),
        );
    };
    let data = decompress_entry(info, &cdata);
    let bad = crc32(&data) as i64 != zi_int(info, "CRC");
    make_instance(
        "zipfile.ZipExtFile",
        vec![
            ("_data", new_bytes(data)),
            ("_pos", MbValue::from_int(0)),
            ("_bad", MbValue::from_bool(bad)),
            ("name", new_str(&name)),
        ],
    )
}

unsafe extern "C" fn extfile_read(self_v: MbValue, args: MbValue) -> MbValue {
    let items = seq_items(args);
    let n = items.first().and_then(|v| v.as_int());
    let data = get_field(self_v, "_data")
        .and_then(extract_bytes)
        .unwrap_or_default();
    let pos = get_field(self_v, "_pos")
        .and_then(|v| v.as_int())
        .unwrap_or(0) as usize;
    let bad = get_field(self_v, "_bad")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let end = match n {
        Some(k) if k >= 0 => (pos + k as usize).min(data.len()),
        _ => data.len(),
    };
    if pos >= data.len() {
        // Exhausted: a bad CRC surfaces at end-of-stream like CPython.
        if bad {
            let name = get_field(self_v, "name")
                .and_then(extract_str)
                .unwrap_or_default();
            return raise_str(
                "zipfile.BadZipFile",
                &format!("Bad CRC-32 for file {name:?}"),
            );
        }
        return new_bytes(Vec::new());
    }
    set_field(self_v, "_pos", MbValue::from_int(end as i64));
    if end >= data.len() && bad && n.is_none() {
        let name = get_field(self_v, "name")
            .and_then(extract_str)
            .unwrap_or_default();
        return raise_str(
            "zipfile.BadZipFile",
            &format!("Bad CRC-32 for file {name:?}"),
        );
    }
    new_bytes(data[pos..end].to_vec())
}

unsafe extern "C" fn extfile_enter(self_v: MbValue, _args: MbValue) -> MbValue {
    self_v
}

unsafe extern "C" fn extfile_exit(_self_v: MbValue, _args: MbValue) -> MbValue {
    MbValue::from_bool(false)
}

unsafe extern "C" fn writefile_write(self_v: MbValue, args: MbValue) -> MbValue {
    let items = seq_items(args);
    let chunk = items
        .first()
        .copied()
        .and_then(extract_bytes)
        .or_else(|| {
            items
                .first()
                .copied()
                .and_then(extract_str)
                .map(|s| s.into_bytes())
        })
        .unwrap_or_default();
    let mut acc = get_field(self_v, "_chunks")
        .and_then(extract_bytes)
        .unwrap_or_default();
    let n = chunk.len();
    acc.extend_from_slice(&chunk);
    set_field(self_v, "_chunks", new_bytes(acc));
    MbValue::from_int(n as i64)
}

unsafe extern "C" fn writefile_close(self_v: MbValue, _args: MbValue) -> MbValue {
    if get_field(self_v, "_done").and_then(|v| v.as_bool()) == Some(true) {
        return MbValue::none();
    }
    set_field(self_v, "_done", MbValue::from_bool(true));
    let zf = get_field(self_v, "_zf").unwrap_or_else(MbValue::none);
    let name = get_field(self_v, "_name").unwrap_or_else(MbValue::none);
    let data = get_field(self_v, "_chunks")
        .and_then(extract_bytes)
        .unwrap_or_default();
    let force64 = get_field(self_v, "_force64")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    writestr_core(zf, name, data, force64)
}

unsafe extern "C" fn writefile_exit(self_v: MbValue, args: MbValue) -> MbValue {
    unsafe { writefile_close(self_v, args) };
    MbValue::from_bool(false)
}

// ── extract / close ──

unsafe extern "C" fn method_extract(self_v: MbValue, args: MbValue) -> MbValue {
    let items = seq_items(args);
    let name = items
        .first()
        .copied()
        .and_then(extract_str)
        .unwrap_or_default();
    let dest = items
        .get(1)
        .copied()
        .and_then(extract_str)
        .unwrap_or_else(|| ".".to_string());
    extract_one(self_v, &name, &dest)
}

fn extract_one(zf: MbValue, name: &str, dest: &str) -> MbValue {
    let Some((info, cdata)) = find_entry_payload(zf, name) else {
        return raise_str(
            "KeyError",
            &format!("There is no item named {name:?} in the archive"),
        );
    };
    let data = decompress_entry(info, &cdata);
    // Sanitize: strip leading slashes and drop '..' components.
    let clean: Vec<&str> = name
        .split('/')
        .filter(|c| !c.is_empty() && *c != "..")
        .collect();
    let mut path = std::path::PathBuf::from(dest);
    for c in &clean {
        path.push(c);
    }
    if name.ends_with('/') {
        let _ = std::fs::create_dir_all(&path);
    } else {
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let _ = std::fs::write(&path, data);
    }
    new_str(&path.to_string_lossy())
}

unsafe extern "C" fn method_extractall(self_v: MbValue, args: MbValue) -> MbValue {
    let items = seq_items(args);
    let dest = items
        .first()
        .copied()
        .and_then(extract_str)
        .unwrap_or_else(|| ".".to_string());
    let names: Vec<String> = with_zf(self_v, |st| {
        st.entries
            .iter()
            .map(|e| zi_str(e.info, "filename"))
            .collect()
    })
    .unwrap_or_default();
    for name in names {
        extract_one(self_v, &name, &dest);
    }
    MbValue::none()
}

unsafe extern "C" fn method_close(self_v: MbValue, _args: MbValue) -> MbValue {
    let (mode, already) = with_zf(self_v, |st| (st.mode, st.closed)).unwrap_or(('r', true));
    if already {
        return MbValue::none();
    }
    if mode == 'w' || mode == 'a' {
        // Pick up a comment assigned after construction.
        let comment = get_field(self_v, "comment")
            .and_then(extract_bytes)
            .unwrap_or_default();
        let blob = with_zf(self_v, |st| {
            st.comment = comment.clone();
            serialize_zip(&st.entries, &st.comment)
        })
        .unwrap_or_default();
        let target = get_field(self_v, "_target").unwrap_or_else(MbValue::none);
        target_write(target, blob);
    }
    with_zf(self_v, |st| st.closed = true);
    MbValue::none()
}

unsafe extern "C" fn method_enter(self_v: MbValue, _args: MbValue) -> MbValue {
    self_v
}

unsafe extern "C" fn method_exit(self_v: MbValue, args: MbValue) -> MbValue {
    unsafe { method_close(self_v, args) };
    MbValue::from_bool(false)
}

// ── module-level ──

unsafe extern "C" fn d_is_zipfile(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { arg_slice(args_ptr, nargs) };
    let target = a.first().copied().unwrap_or_else(MbValue::none);
    let buf = target_read(target).unwrap_or_default();
    MbValue::from_bool(parse_zip(&buf).is_ok())
}

unsafe extern "C" fn d_zipfile_badzipfile(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    make_instance("zipfile.BadZipFile", vec![])
}

// ── Registration ──

pub fn register() {
    register_zip_classes();

    let mut attrs: HashMap<String, MbValue> = HashMap::new();
    let dispatchers: Vec<(&str, usize)> = vec![
        ("ZipFile", d_zipfile_new as *const () as usize),
        ("ZipInfo", d_zipinfo_new as *const () as usize),
        ("is_zipfile", d_is_zipfile as *const () as usize),
        ("BadZipFile", d_zipfile_badzipfile as *const () as usize),
        ("BadZipfile", d_zipfile_badzipfile as *const () as usize),
        // CPython: `zipfile.error` is an alias for BadZipFile.
        ("error", d_zipfile_badzipfile as *const () as usize),
        ("LargeZipFile", d_zipfile_badzipfile as *const () as usize),
        ("Path", d_zipfile_badzipfile as *const () as usize),
        ("PyZipFile", d_zipfile_new as *const () as usize),
    ];
    for (name, addr) in dispatchers {
        attrs.insert(name.to_string(), MbValue::from_func(addr));
        super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
            s.borrow_mut().insert(addr as u64);
        });
    }
    attrs.insert("ZIP_STORED".into(), MbValue::from_int(0));
    attrs.insert("ZIP_DEFLATED".into(), MbValue::from_int(8));
    attrs.insert("ZIP_BZIP2".into(), MbValue::from_int(12));
    attrs.insert("ZIP_LZMA".into(), MbValue::from_int(14));
    attrs.insert("ZIP64_VERSION".into(), MbValue::from_int(45));
    attrs.insert("DEFAULT_VERSION".into(), MbValue::from_int(20));
    attrs.insert("ZIP64_LIMIT".into(), MbValue::from_int((1i64 << 31) - 1));
    attrs.insert("ZIP_MAX_COMMENT".into(), MbValue::from_int(65535));
    // Fixed on-disk structure sizes (CPython zipfile module constants).
    attrs.insert("sizeEndCentDir".into(), MbValue::from_int(22));
    attrs.insert("sizeCentralDir".into(), MbValue::from_int(46));
    attrs.insert("sizeFileHeader".into(), MbValue::from_int(30));
    attrs.insert("sizeEndCentDir64".into(), MbValue::from_int(56));
    attrs.insert("sizeEndCentDir64Locator".into(), MbValue::from_int(20));

    // `except zipfile.BadZipFile` resolves the constructor func to the raised
    // type name through NATIVE_TYPE_NAMES.
    super::super::module::NATIVE_TYPE_NAMES.with(|m| {
        let mut map = m.borrow_mut();
        map.insert(
            d_zipfile_badzipfile as *const () as usize as u64,
            "zipfile.BadZipFile".to_string(),
        );
        map.insert(
            d_zipinfo_new as *const () as usize as u64,
            "zipfile.ZipInfo".to_string(),
        );
        map.insert(
            d_zipfile_new as *const () as usize as u64,
            ZIPFILE_CLASS.to_string(),
        );
    });

    super::register_module("zipfile", attrs);
}

fn register_zip_classes() {
    use std::collections::HashMap as Map;
    let var = |addr: usize| {
        super::super::module::register_variadic_func(addr as u64);
        MbValue::from_func(addr)
    };

    let mut zf: Map<String, MbValue> = Map::new();
    for (name, addr) in [
        ("writestr", method_writestr as *const () as usize),
        ("write", method_write as *const () as usize),
        ("read", method_read as *const () as usize),
        ("namelist", method_namelist as *const () as usize),
        ("infolist", method_infolist as *const () as usize),
        ("getinfo", method_getinfo as *const () as usize),
        ("testzip", method_testzip as *const () as usize),
        ("open", method_open as *const () as usize),
        ("extract", method_extract as *const () as usize),
        ("extractall", method_extractall as *const () as usize),
        ("close", method_close as *const () as usize),
        ("__enter__", method_enter as *const () as usize),
        ("__exit__", method_exit as *const () as usize),
    ] {
        zf.insert(name.to_string(), var(addr));
    }
    super::super::class::mb_class_register(ZIPFILE_CLASS, vec![], zf);

    let mut zi: Map<String, MbValue> = Map::new();
    zi.insert("is_dir".into(), var(zipinfo_is_dir as *const () as usize));
    zi.insert("__repr__".into(), var(zipinfo_repr as *const () as usize));
    zi.insert(
        "from_file".into(),
        MbValue::from_func(d_zipinfo_from_file as *const () as usize),
    );
    super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
        s.borrow_mut()
            .insert(d_zipinfo_from_file as *const () as usize as u64);
    });
    super::super::class::mb_class_register(ZIPINFO_CLASS, vec![], zi);
    // The qualified name is what NATIVE_TYPE_NAMES maps to for the gate.
    let mut zi2: Map<String, MbValue> = Map::new();
    zi2.insert(
        "from_file".into(),
        MbValue::from_func(d_zipinfo_from_file as *const () as usize),
    );
    super::super::class::mb_class_register("zipfile.ZipInfo", vec![], zi2);

    let mut ext: Map<String, MbValue> = Map::new();
    ext.insert("read".into(), var(extfile_read as *const () as usize));
    ext.insert("__enter__".into(), var(extfile_enter as *const () as usize));
    ext.insert("__exit__".into(), var(extfile_exit as *const () as usize));
    super::super::class::mb_class_register("zipfile.ZipExtFile", vec![], ext);

    let mut wf: Map<String, MbValue> = Map::new();
    wf.insert("write".into(), var(writefile_write as *const () as usize));
    wf.insert("close".into(), var(writefile_close as *const () as usize));
    wf.insert("__enter__".into(), var(extfile_enter as *const () as usize));
    wf.insert("__exit__".into(), var(writefile_exit as *const () as usize));
    super::super::class::mb_class_register("zipfile._ZipWriteFile", vec![], wf);
}

// Legacy public helpers kept for callers/tests.

pub fn mb_zipfile_new(file: MbValue, mode: MbValue) -> MbValue {
    let args = [file, mode];
    unsafe { d_zipfile_new(args.as_ptr(), 2) }
}

pub fn mb_zipfile_is_zipfile(path: MbValue) -> MbValue {
    let args = [path];
    unsafe { d_is_zipfile(args.as_ptr(), 1) }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(val: &str) -> MbValue {
        new_str(val)
    }

    fn list(items: Vec<MbValue>) -> MbValue {
        MbValue::from_ptr(MbObject::new_list(items))
    }

    #[test]
    fn test_roundtrip_serialize_parse() {
        register();
        let zi = make_zipinfo("a.txt", &[1980, 1, 1, 0, 0, 0]);
        let data = b"hello world".to_vec();
        set_field(zi, "file_size", MbValue::from_int(data.len() as i64));
        set_field(zi, "CRC", MbValue::from_int(crc32(&data) as i64));
        let entries = vec![Entry {
            info: zi,
            cdata: data.clone(),
            force_zip64: false,
        }];
        let blob = serialize_zip(&entries, b"cmt");
        let (parsed, comment) = parse_zip(&blob).expect("parse back");
        assert_eq!(parsed.len(), 1);
        assert_eq!(comment, b"cmt");
        assert_eq!(zi_str(parsed[0].info, "filename"), "a.txt");
        assert_eq!(parsed[0].cdata, data);
    }

    #[test]
    fn test_deflate_roundtrip() {
        let data = b"aaaa".repeat(100);
        let c = deflate(&data);
        assert!(c.len() < data.len());
        assert_eq!(inflate(&c), data);
    }

    #[test]
    fn test_writestr_read_via_methods() {
        register();
        // In-memory BytesIO-like stand-in.
        let fileobj = make_instance(
            "BytesIO",
            vec![
                ("_buffer", new_bytes(Vec::new())),
                ("_pos", MbValue::from_int(0)),
            ],
        );
        let zf = mb_zipfile_new(fileobj, s("w"));
        unsafe {
            method_writestr(zf, list(vec![s("x.txt"), s("payload")]));
            let got = method_read(zf, list(vec![s("x.txt")]));
            assert_eq!(extract_bytes(got).as_deref(), Some(b"payload".as_ref()));
            method_close(zf, list(vec![]));
        }
        // Reopen from the serialized bytes.
        let zf2 = mb_zipfile_new(fileobj, s("r"));
        unsafe {
            let got = method_read(zf2, list(vec![s("x.txt")]));
            assert_eq!(extract_bytes(got).as_deref(), Some(b"payload".as_ref()));
        }
    }

    #[test]
    fn test_zipinfo_defaults() {
        register();
        let zi = unsafe { d_zipinfo_new(std::ptr::null(), 0) };
        assert_eq!(zi_str(zi, "filename"), "NoName");
        assert_eq!(zi_int(zi, "compress_type"), 0);
        assert_eq!(zi_int(zi, "file_size"), 0);
    }
}
