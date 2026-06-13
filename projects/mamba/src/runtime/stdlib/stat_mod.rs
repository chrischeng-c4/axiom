/// stat module for Mamba.
///
/// Implements Python 3.12 `stat` stdlib: file type constants, permission constants,
/// stat result field index constants, file type test functions, and permission helpers.
/// All constant values match CPython 3.12 / POSIX exactly.
use std::collections::HashMap;
use super::super::value::MbValue;
use super::super::rc::MbObject;

macro_rules! dispatch_unary {
    ($name:ident, $fn:ident) => {
        unsafe extern "C" fn $name(args_ptr: *const MbValue, nargs: usize) -> MbValue {
            let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
            let arg = a.get(0).copied().unwrap_or_else(MbValue::none);
            // Every stat mode helper operates on an integer mode (`mode & ...`);
            // a non-int argument (e.g. S_ISDIR("x")) is a TypeError, not a
            // silent 0-mode.
            if arg.as_int_pyint().is_none() {
                super::super::exception::mb_raise(
                    MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
                    MbValue::from_ptr(MbObject::new_str(format!(
                        "unsupported operand type(s) for &: '{}' and 'int'",
                        super::super::builtins::value_type_name(arg)
                    ))),
                );
                return MbValue::none();
            }
            $fn(arg)
        }
    };
}

dispatch_unary!(dispatch_s_isdir, mb_stat_s_isdir);
dispatch_unary!(dispatch_s_ischr, mb_stat_s_ischr);
dispatch_unary!(dispatch_s_isblk, mb_stat_s_isblk);
dispatch_unary!(dispatch_s_isreg, mb_stat_s_isreg);
dispatch_unary!(dispatch_s_isfifo, mb_stat_s_isfifo);
dispatch_unary!(dispatch_s_islnk, mb_stat_s_islnk);
dispatch_unary!(dispatch_s_issock, mb_stat_s_issock);
dispatch_unary!(dispatch_s_iswht, mb_stat_s_iswht);
dispatch_unary!(dispatch_s_isdoor, mb_stat_s_isdoor);
dispatch_unary!(dispatch_s_isport, mb_stat_s_isport);
dispatch_unary!(dispatch_s_imode, mb_stat_s_imode);
dispatch_unary!(dispatch_s_ifmt_fn, mb_stat_s_ifmt_fn);
dispatch_unary!(dispatch_filemode, mb_stat_filemode);

// ── File type constants (from <sys/stat.h>) ──

pub const S_IFMT: i64 = 0o170000;
pub const S_IFSOCK: i64 = 0o140000;
pub const S_IFLNK: i64 = 0o120000;
pub const S_IFREG: i64 = 0o100000;
pub const S_IFBLK: i64 = 0o060000;
pub const S_IFDIR: i64 = 0o040000;
pub const S_IFCHR: i64 = 0o020000;
pub const S_IFIFO: i64 = 0o010000;

// ── File mode/permission constants ──

pub const S_ISUID: i64 = 0o4000;
pub const S_ISGID: i64 = 0o2000;
pub const S_ISVTX: i64 = 0o1000;

pub const S_IRWXU: i64 = 0o700;
pub const S_IRUSR: i64 = 0o400;
pub const S_IWUSR: i64 = 0o200;
pub const S_IXUSR: i64 = 0o100;

pub const S_IRWXG: i64 = 0o070;
pub const S_IRGRP: i64 = 0o040;
pub const S_IWGRP: i64 = 0o020;
pub const S_IXGRP: i64 = 0o010;

pub const S_IRWXO: i64 = 0o007;
pub const S_IROTH: i64 = 0o004;
pub const S_IWOTH: i64 = 0o002;
pub const S_IXOTH: i64 = 0o001;

// ── stat result field index constants ──

pub const ST_MODE: i64 = 0;
pub const ST_INO: i64 = 1;
pub const ST_DEV: i64 = 2;
pub const ST_NLINK: i64 = 3;
pub const ST_UID: i64 = 4;
pub const ST_GID: i64 = 5;
pub const ST_SIZE: i64 = 6;
pub const ST_ATIME: i64 = 7;
pub const ST_MTIME: i64 = 8;
pub const ST_CTIME: i64 = 9;

/// All integer constants ordered for registration.
///
/// Note: `S_IFMT` is intentionally absent here. In CPython `stat.S_IFMT` is the
/// *function* `S_IFMT(mode)` (the file-type mask `0o170000` is internal only),
/// so it is registered as a callable in `register()`, not as an int constant.
const STAT_CONSTANTS: &[(&str, i64)] = &[
    // File type constants
    ("S_IFSOCK", S_IFSOCK),
    ("S_IFLNK", S_IFLNK),
    ("S_IFREG", S_IFREG),
    ("S_IFBLK", S_IFBLK),
    ("S_IFDIR", S_IFDIR),
    ("S_IFCHR", S_IFCHR),
    ("S_IFIFO", S_IFIFO),
    // Permission constants
    ("S_ISUID", S_ISUID),
    ("S_ISGID", S_ISGID),
    ("S_ISVTX", S_ISVTX),
    ("S_IRWXU", S_IRWXU),
    ("S_IRUSR", S_IRUSR),
    ("S_IWUSR", S_IWUSR),
    ("S_IXUSR", S_IXUSR),
    ("S_IRWXG", S_IRWXG),
    ("S_IRGRP", S_IRGRP),
    ("S_IWGRP", S_IWGRP),
    ("S_IXGRP", S_IXGRP),
    ("S_IRWXO", S_IRWXO),
    ("S_IROTH", S_IROTH),
    ("S_IWOTH", S_IWOTH),
    ("S_IXOTH", S_IXOTH),
    // stat result field indices
    ("ST_MODE", ST_MODE),
    ("ST_INO", ST_INO),
    ("ST_DEV", ST_DEV),
    ("ST_NLINK", ST_NLINK),
    ("ST_UID", ST_UID),
    ("ST_GID", ST_GID),
    ("ST_SIZE", ST_SIZE),
    ("ST_ATIME", ST_ATIME),
    ("ST_MTIME", ST_MTIME),
    ("ST_CTIME", ST_CTIME),
];

pub fn register() {
    let mut attrs = HashMap::new();
    // Register all integer constants.
    for (name, value) in STAT_CONSTANTS {
        attrs.insert(name.to_string(), MbValue::from_int(*value));
    }
    // Register callable function symbols.
    let dispatchers: Vec<(&str, usize)> = vec![
        ("S_ISDIR", dispatch_s_isdir as usize),
        ("S_ISCHR", dispatch_s_ischr as usize),
        ("S_ISBLK", dispatch_s_isblk as usize),
        ("S_ISREG", dispatch_s_isreg as usize),
        ("S_ISFIFO", dispatch_s_isfifo as usize),
        ("S_ISLNK", dispatch_s_islnk as usize),
        ("S_ISSOCK", dispatch_s_issock as usize),
        ("S_ISWHT", dispatch_s_iswht as usize),
        ("S_ISDOOR", dispatch_s_isdoor as usize),
        ("S_ISPORT", dispatch_s_isport as usize),
        ("S_IMODE", dispatch_s_imode as usize),
        // In CPython `stat.S_IFMT` is the function `S_IFMT(mode)` (the mask
        // 0o170000 is internal only), so register the callable under `S_IFMT`.
        ("S_IFMT", dispatch_s_ifmt_fn as usize),
        ("filemode", dispatch_filemode as usize),
    ];
    for (name, addr) in dispatchers {
        attrs.insert(name.to_string(), MbValue::from_func(addr));
        super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
            s.borrow_mut().insert(addr as u64);
        });
    }
        // surface: missing CPython module constants (auto-added)
    attrs.insert("FILE_ATTRIBUTE_ARCHIVE".into(), MbValue::from_int(32));
    attrs.insert("FILE_ATTRIBUTE_COMPRESSED".into(), MbValue::from_int(2048));
    attrs.insert("FILE_ATTRIBUTE_DEVICE".into(), MbValue::from_int(64));
    attrs.insert("FILE_ATTRIBUTE_DIRECTORY".into(), MbValue::from_int(16));
    attrs.insert("FILE_ATTRIBUTE_ENCRYPTED".into(), MbValue::from_int(16384));
    attrs.insert("FILE_ATTRIBUTE_HIDDEN".into(), MbValue::from_int(2));
    attrs.insert("FILE_ATTRIBUTE_INTEGRITY_STREAM".into(), MbValue::from_int(32768));
    attrs.insert("FILE_ATTRIBUTE_NORMAL".into(), MbValue::from_int(128));
    attrs.insert("FILE_ATTRIBUTE_NOT_CONTENT_INDEXED".into(), MbValue::from_int(8192));
    attrs.insert("FILE_ATTRIBUTE_NO_SCRUB_DATA".into(), MbValue::from_int(131072));
    attrs.insert("FILE_ATTRIBUTE_OFFLINE".into(), MbValue::from_int(4096));
    attrs.insert("FILE_ATTRIBUTE_READONLY".into(), MbValue::from_int(1));
    attrs.insert("FILE_ATTRIBUTE_REPARSE_POINT".into(), MbValue::from_int(1024));
    attrs.insert("FILE_ATTRIBUTE_SPARSE_FILE".into(), MbValue::from_int(512));
    attrs.insert("FILE_ATTRIBUTE_SYSTEM".into(), MbValue::from_int(4));
    attrs.insert("FILE_ATTRIBUTE_TEMPORARY".into(), MbValue::from_int(256));
    attrs.insert("FILE_ATTRIBUTE_VIRTUAL".into(), MbValue::from_int(65536));
    attrs.insert("SF_APPEND".into(), MbValue::from_int(262144));
    attrs.insert("SF_ARCHIVED".into(), MbValue::from_int(65536));
    attrs.insert("SF_IMMUTABLE".into(), MbValue::from_int(131072));
    attrs.insert("SF_NOUNLINK".into(), MbValue::from_int(1048576));
    attrs.insert("SF_SNAPSHOT".into(), MbValue::from_int(2097152));
    attrs.insert("S_ENFMT".into(), MbValue::from_int(1024));
    attrs.insert("S_IEXEC".into(), MbValue::from_int(64));
    attrs.insert("S_IFDOOR".into(), MbValue::from_int(0));
    attrs.insert("S_IFPORT".into(), MbValue::from_int(0));
    attrs.insert("S_IFWHT".into(), MbValue::from_int(57344));
    attrs.insert("S_IREAD".into(), MbValue::from_int(256));
    attrs.insert("S_IWRITE".into(), MbValue::from_int(128));
    attrs.insert("UF_APPEND".into(), MbValue::from_int(4));
    attrs.insert("UF_COMPRESSED".into(), MbValue::from_int(32));
    attrs.insert("UF_HIDDEN".into(), MbValue::from_int(32768));
    attrs.insert("UF_IMMUTABLE".into(), MbValue::from_int(2));
    attrs.insert("UF_NODUMP".into(), MbValue::from_int(1));
    attrs.insert("UF_NOUNLINK".into(), MbValue::from_int(16));
    attrs.insert("UF_OPAQUE".into(), MbValue::from_int(8));
    super::register_module("stat", attrs);
}

// ── File type test functions ──

/// stat.S_ISDIR(mode) -> bool — True if mode indicates a directory.
pub fn mb_stat_s_isdir(mode: MbValue) -> MbValue {
    let m = mode.as_int().unwrap_or(0);
    MbValue::from_bool((m & S_IFMT) == S_IFDIR)
}

/// stat.S_ISCHR(mode) -> bool — True if mode indicates a character device.
pub fn mb_stat_s_ischr(mode: MbValue) -> MbValue {
    let m = mode.as_int().unwrap_or(0);
    MbValue::from_bool((m & S_IFMT) == S_IFCHR)
}

/// stat.S_ISBLK(mode) -> bool — True if mode indicates a block device.
pub fn mb_stat_s_isblk(mode: MbValue) -> MbValue {
    let m = mode.as_int().unwrap_or(0);
    MbValue::from_bool((m & S_IFMT) == S_IFBLK)
}

/// stat.S_ISREG(mode) -> bool — True if mode indicates a regular file.
pub fn mb_stat_s_isreg(mode: MbValue) -> MbValue {
    let m = mode.as_int().unwrap_or(0);
    MbValue::from_bool((m & S_IFMT) == S_IFREG)
}

/// stat.S_ISFIFO(mode) -> bool — True if mode indicates a FIFO (named pipe).
pub fn mb_stat_s_isfifo(mode: MbValue) -> MbValue {
    let m = mode.as_int().unwrap_or(0);
    MbValue::from_bool((m & S_IFMT) == S_IFIFO)
}

/// stat.S_ISLNK(mode) -> bool — True if mode indicates a symbolic link.
pub fn mb_stat_s_islnk(mode: MbValue) -> MbValue {
    let m = mode.as_int().unwrap_or(0);
    MbValue::from_bool((m & S_IFMT) == S_IFLNK)
}

/// stat.S_ISSOCK(mode) -> bool — True if mode indicates a socket.
pub fn mb_stat_s_issock(mode: MbValue) -> MbValue {
    let m = mode.as_int().unwrap_or(0);
    MbValue::from_bool((m & S_IFMT) == S_IFSOCK)
}

/// stat.S_ISWHT(mode) -> bool — True if mode indicates a whiteout (BSD).
pub fn mb_stat_s_iswht(mode: MbValue) -> MbValue {
    let m = mode.as_int().unwrap_or(0);
    // S_IFWHT == 0o160000 (CPython 3.12).
    MbValue::from_bool((m & S_IFMT) == 0o160000)
}

/// stat.S_ISDOOR(mode) -> bool — True if mode indicates a door (Solaris).
pub fn mb_stat_s_isdoor(mode: MbValue) -> MbValue {
    let m = mode.as_int().unwrap_or(0);
    // S_IFDOOR == 0 on non-Solaris CPython builds; matches CPython 3.12.
    MbValue::from_bool((m & S_IFMT) == 0)
}

/// stat.S_ISPORT(mode) -> bool — True if mode indicates an event port (Solaris).
pub fn mb_stat_s_isport(mode: MbValue) -> MbValue {
    let m = mode.as_int().unwrap_or(0);
    // S_IFPORT == 0 on non-Solaris CPython builds; matches CPython 3.12.
    MbValue::from_bool((m & S_IFMT) == 0)
}

// ── Permission helpers ──

/// stat.S_IMODE(mode) -> int — Extract the permission bits from a mode value.
/// Returns mode & 0o7777 (lower 12 bits: permissions + setuid/setgid/sticky).
pub fn mb_stat_s_imode(mode: MbValue) -> MbValue {
    let m = mode.as_int().unwrap_or(0);
    MbValue::from_int(m & 0o7777)
}

/// stat.S_IFMT(mode) -> int — Extract the file type bits from a mode value.
/// Returns mode & 0o170000.
/// Named _fn to avoid a name collision with the S_IFMT integer constant.
pub fn mb_stat_s_ifmt_fn(mode: MbValue) -> MbValue {
    let m = mode.as_int().unwrap_or(0);
    MbValue::from_int(m & S_IFMT)
}

/// stat.filemode(mode) -> str — Convert a mode integer to a 10-character string.
///
/// Character positions:
///   0:   file type  (d=dir, l=symlink, c=char, b=block, p=fifo, s=socket, -=regular)
///   1-3: owner rwx  (s/S if setuid bit set, replaces x/-)
///   4-6: group rwx  (s/S if setgid bit set, replaces x/-)
///   7-9: other rwx  (t/T if sticky bit set, replaces x/-)
pub fn mb_stat_filemode(mode: MbValue) -> MbValue {
    let m = mode.as_int().unwrap_or(0);
    let mut result = [b'-'; 10];

    // Character 0: file type.
    // CPython renders a regular file (S_IFREG) as '-' and any *unrecognized*
    // type — including mode 0 — as '?', matching its `_filemode_table` default.
    result[0] = match m & S_IFMT {
        x if x == S_IFDIR  => b'd',
        x if x == S_IFLNK  => b'l',
        x if x == S_IFCHR  => b'c',
        x if x == S_IFBLK  => b'b',
        x if x == S_IFIFO  => b'p',
        x if x == S_IFSOCK => b's',
        x if x == S_IFREG  => b'-',
        _                   => b'?',
    };

    // Characters 1-3: owner permissions
    if m & S_IRUSR != 0 { result[1] = b'r'; }
    if m & S_IWUSR != 0 { result[2] = b'w'; }
    result[3] = if m & S_ISUID != 0 {
        if m & S_IXUSR != 0 { b's' } else { b'S' }
    } else if m & S_IXUSR != 0 {
        b'x'
    } else {
        b'-'
    };

    // Characters 4-6: group permissions
    if m & S_IRGRP != 0 { result[4] = b'r'; }
    if m & S_IWGRP != 0 { result[5] = b'w'; }
    result[6] = if m & S_ISGID != 0 {
        if m & S_IXGRP != 0 { b's' } else { b'S' }
    } else if m & S_IXGRP != 0 {
        b'x'
    } else {
        b'-'
    };

    // Characters 7-9: other permissions
    if m & S_IROTH != 0 { result[7] = b'r'; }
    if m & S_IWOTH != 0 { result[8] = b'w'; }
    result[9] = if m & S_ISVTX != 0 {
        if m & S_IXOTH != 0 { b't' } else { b'T' }
    } else if m & S_IXOTH != 0 {
        b'x'
    } else {
        b'-'
    };

    let s = String::from_utf8(result.to_vec()).expect("filemode output is always ASCII");
    MbValue::from_ptr(MbObject::new_str(s))
}

/// Extract a Rust String from an MbValue that holds a heap string object.
/// Returns an empty string if the value is not a string.
#[allow(dead_code)]
fn extract_str(val: MbValue) -> String {
    val.as_ptr()
        .and_then(|ptr| unsafe {
            use super::super::rc::ObjData;
            if let ObjData::Str(ref s) = (*ptr).data {
                Some(s.clone())
            } else {
                None
            }
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    // REQ: R1
    #[test]
    fn test_constants() {
        // Spot-check file type constants.
        assert_eq!(S_IFDIR, 0o040000, "S_IFDIR must be 0o040000");
        assert_eq!(S_IFREG, 0o100000, "S_IFREG must be 0o100000");
        assert_eq!(S_IFLNK, 0o120000, "S_IFLNK must be 0o120000");
        assert_eq!(S_IFMT,  0o170000, "S_IFMT must be 0o170000");
        // Spot-check permission constants.
        assert_eq!(S_IRUSR, 0o400, "S_IRUSR must be 0o400");
        assert_eq!(S_IWUSR, 0o200, "S_IWUSR must be 0o200");
        assert_eq!(S_IXUSR, 0o100, "S_IXUSR must be 0o100");
        // Spot-check stat result field indices.
        assert_eq!(ST_MODE, 0, "ST_MODE must be 0");
        assert_eq!(ST_SIZE, 6, "ST_SIZE must be 6");
        assert_eq!(ST_CTIME, 9, "ST_CTIME must be 9");
    }

    // REQ: R2
    #[test]
    fn test_s_isdir() {
        // Directory mode 0o040755 — S_ISDIR should be true.
        let dir_mode = MbValue::from_int(0o040755);
        assert_eq!(
            mb_stat_s_isdir(dir_mode).as_bool(),
            Some(true),
            "S_ISDIR(0o040755) must be true"
        );
        // Regular file mode 0o100644 — S_ISDIR should be false.
        let reg_mode = MbValue::from_int(0o100644);
        assert_eq!(
            mb_stat_s_isdir(reg_mode).as_bool(),
            Some(false),
            "S_ISDIR(0o100644) must be false"
        );
    }

    // REQ: R2
    #[test]
    fn test_s_isreg() {
        // Regular file mode 0o100644 — S_ISREG should be true.
        let reg_mode = MbValue::from_int(0o100644);
        assert_eq!(
            mb_stat_s_isreg(reg_mode).as_bool(),
            Some(true),
            "S_ISREG(0o100644) must be true"
        );
        // Directory mode — S_ISREG should be false.
        let dir_mode = MbValue::from_int(0o040755);
        assert_eq!(
            mb_stat_s_isreg(dir_mode).as_bool(),
            Some(false),
            "S_ISREG(0o040755) must be false"
        );
    }

    // REQ: R2
    #[test]
    fn test_s_islnk() {
        // Symlink mode 0o120777 — S_ISLNK should be true.
        let lnk_mode = MbValue::from_int(0o120777);
        assert_eq!(
            mb_stat_s_islnk(lnk_mode).as_bool(),
            Some(true),
            "S_ISLNK(0o120777) must be true"
        );
        // Regular file mode — S_ISLNK should be false.
        let reg_mode = MbValue::from_int(0o100644);
        assert_eq!(
            mb_stat_s_islnk(reg_mode).as_bool(),
            Some(false),
            "S_ISLNK(0o100644) must be false"
        );
    }

    // REQ: R3
    #[test]
    fn test_s_imode() {
        // 0o100755 — S_IMODE should return 0o755.
        let mode = MbValue::from_int(0o100755);
        assert_eq!(
            mb_stat_s_imode(mode).as_int(),
            Some(0o755),
            "S_IMODE(0o100755) must be 0o755"
        );
        // With setuid bit: 0o104755 — S_IMODE should return 0o4755.
        let setuid_mode = MbValue::from_int(0o104755);
        assert_eq!(
            mb_stat_s_imode(setuid_mode).as_int(),
            Some(0o4755),
            "S_IMODE(0o104755) must be 0o4755"
        );
    }

    // REQ: R3
    #[test]
    fn test_s_ifmt() {
        // 0o100644 — S_IFMT should return 0o100000 (S_IFREG).
        let mode = MbValue::from_int(0o100644);
        assert_eq!(
            mb_stat_s_ifmt_fn(mode).as_int(),
            Some(0o100000),
            "S_IFMT(0o100644) must be 0o100000"
        );
        // 0o040755 — S_IFMT should return 0o040000 (S_IFDIR).
        let dir_mode = MbValue::from_int(0o040755);
        assert_eq!(
            mb_stat_s_ifmt_fn(dir_mode).as_int(),
            Some(0o040000),
            "S_IFMT(0o040755) must be 0o040000"
        );
    }

    // REQ: R4
    #[test]
    fn test_filemode() {
        // Regular file with rwxr-xr-x (0o100755) → "-rwxr-xr-x"
        let mode = MbValue::from_int(0o100755);
        let result = mb_stat_filemode(mode);
        let s = extract_str(result);
        assert_eq!(s, "-rwxr-xr-x", "filemode(0o100755) must be '-rwxr-xr-x'");
    }

    // REQ: R4
    #[test]
    fn test_filemode_directory() {
        // Directory with rwxr-xr-x (0o040755) → "drwxr-xr-x"
        let mode = MbValue::from_int(0o040755);
        let result = mb_stat_filemode(mode);
        let s = extract_str(result);
        assert_eq!(s, "drwxr-xr-x", "filemode(0o040755) must be 'drwxr-xr-x'");
    }

    // REQ: R4
    #[test]
    fn test_filemode_symlink() {
        // Symlink with rwxrwxrwx (0o120777) → "lrwxrwxrwx"
        let mode = MbValue::from_int(0o120777);
        let result = mb_stat_filemode(mode);
        let s = extract_str(result);
        assert_eq!(s, "lrwxrwxrwx", "filemode(0o120777) must be 'lrwxrwxrwx'");
    }
}
