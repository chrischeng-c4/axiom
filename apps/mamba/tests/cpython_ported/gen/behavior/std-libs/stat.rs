use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/std-libs/stat/file_type_constants_are_octal_ints.py`.
#[test]
fn test_gen_behavior_std_libs_stat_file_type_constants_are_octal_ints() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "stat"
# dimension = "behavior"
# case = "file_type_constants_are_octal_ints"
# subject = "stat.S_IFREG"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_stat.py"
# status = "filled"
# ///
"""stat.S_IFREG: file-type constants are ints with the documented octal values: S_IFREG==0o100000, S_IFDIR==0o040000, S_IFLNK==0o120000"""
import stat

for name, value in [("S_IFREG", 0o100000), ("S_IFDIR", 0o040000), ("S_IFLNK", 0o120000)]:
    const = getattr(stat, name)
    assert isinstance(const, int), name
    assert const == value, (name, oct(const), oct(value))

print("file_type_constants_are_octal_ints OK")
"###);
    assert_output(&out, r###"file_type_constants_are_octal_ints OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/stat/filemode_renders_ls_long_style.py`.
#[test]
fn test_gen_behavior_std_libs_stat_filemode_renders_ls_long_style() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "stat"
# dimension = "behavior"
# case = "filemode_renders_ls_long_style"
# subject = "stat.filemode"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_stat.py"
# status = "filled"
# ///
"""stat.filemode: filemode renders an ls -l style string: filemode(0o100644) == '-rw-r--r--' and filemode(0o040755) == 'drwxr-xr-x'"""
import stat

# Regular file, 644 perms -> leading '-' and rw-r--r-- triplets.
assert stat.filemode(0o100644) == "-rw-r--r--", "filemode(reg 644)"
# Directory, 755 perms -> leading 'd' and rwxr-xr-x triplets.
assert stat.filemode(0o040755) == "drwxr-xr-x", "filemode(dir 755)"

print("filemode_renders_ls_long_style OK")
"###);
    assert_output(&out, r###"filemode_renders_ls_long_style OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/stat/filemode_zero_mode_is_forgiving.py`.
#[test]
fn test_gen_behavior_std_libs_stat_filemode_zero_mode_is_forgiving() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "stat"
# dimension = "behavior"
# case = "filemode_zero_mode_is_forgiving"
# subject = "stat.filemode"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""stat.filemode: filemode(0) is forgiving: it returns a 10-char string '?---------' rather than raising"""
import stat

# A zero mode has no recognized file-type bits and no permission bits set;
# filemode renders an unknown-type marker '?' followed by nine dashes.
result = stat.filemode(0)
assert result == "?---------", "filemode(0)"
assert len(result) == 10, "filemode(0) length"

print("filemode_zero_mode_is_forgiving OK")
"###);
    assert_output(&out, r###"filemode_zero_mode_is_forgiving OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/stat/permission_constants_are_octal_ints.py`.
#[test]
fn test_gen_behavior_std_libs_stat_permission_constants_are_octal_ints() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "stat"
# dimension = "behavior"
# case = "permission_constants_are_octal_ints"
# subject = "stat.S_IRUSR"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_stat.py"
# status = "filled"
# ///
"""stat.S_IRUSR: permission constants are ints with the documented octal values: S_IRUSR==0o400, S_IWGRP==0o020, S_IXOTH==0o001, S_IRWXU==0o700, S_ISUID==0o4000"""
import stat

for name, value in [
    ("S_IRUSR", 0o400),
    ("S_IWGRP", 0o020),
    ("S_IXOTH", 0o001),
    ("S_IRWXU", 0o700),
    ("S_ISUID", 0o4000),
]:
    const = getattr(stat, name)
    assert isinstance(const, int), name
    assert const == value, (name, oct(const), oct(value))

print("permission_constants_are_octal_ints OK")
"###);
    assert_output(&out, r###"permission_constants_are_octal_ints OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/stat/s_ifmt_extracts_file_type_bits.py`.
#[test]
fn test_gen_behavior_std_libs_stat_s_ifmt_extracts_file_type_bits() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "stat"
# dimension = "behavior"
# case = "s_ifmt_extracts_file_type_bits"
# subject = "stat.S_IFMT"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_stat.py"
# status = "filled"
# ///
"""stat.S_IFMT: S_IFMT extracts only the file-type bits: S_IFMT(0o100644) == S_IFREG == 0o100000 and S_IFMT(0o040755) == S_IFDIR"""
import stat

# S_IFMT keeps the file-type bits and drops the permission bits.
assert stat.S_IFMT(0o100644) == stat.S_IFREG, "S_IFMT(reg) == S_IFREG"
assert stat.S_IFMT(0o100644) == 0o100000, "S_IFMT(reg) octal"
assert stat.S_IFMT(0o040755) == stat.S_IFDIR, "S_IFMT(dir) == S_IFDIR"
assert stat.S_IFMT(0o120755) == stat.S_IFLNK, "S_IFMT(lnk) == S_IFLNK"

print("s_ifmt_extracts_file_type_bits OK")
"###);
    assert_output(&out, r###"s_ifmt_extracts_file_type_bits OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/stat/s_imode_strips_file_type_bits.py`.
#[test]
fn test_gen_behavior_std_libs_stat_s_imode_strips_file_type_bits() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "stat"
# dimension = "behavior"
# case = "s_imode_strips_file_type_bits"
# subject = "stat.S_IMODE"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""stat.S_IMODE: S_IMODE strips the file-type bits and keeps only the permission bits: S_IMODE(0o100755) == 0o755"""
import stat

# A regular file (0o100000) with rwxr-xr-x perms keeps only the 0o755 perms.
assert stat.S_IMODE(0o100755) == 0o755, "S_IMODE(0o100755)"
assert oct(stat.S_IMODE(0o100755)) == "0o755", "oct(S_IMODE(0o100755))"
# A directory (0o040000) with 700 perms keeps only 0o700.
assert stat.S_IMODE(0o040700) == 0o700, "S_IMODE(0o040700)"

print("s_imode_strips_file_type_bits OK")
"###);
    assert_output(&out, r###"s_imode_strips_file_type_bits OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/stat/s_is_predicates_match_file_type.py`.
#[test]
fn test_gen_behavior_std_libs_stat_s_is_predicates_match_file_type() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "stat"
# dimension = "behavior"
# case = "s_is_predicates_match_file_type"
# subject = "stat.S_ISDIR"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_stat.py"
# status = "filled"
# ///
"""stat.S_ISDIR: each S_IS* predicate returns True only for its own file-type mode and False for a regular-file mode: S_ISDIR(0o040755), S_ISREG(0o100644), S_ISLNK(0o120755), S_ISFIFO(0o010000), S_ISCHR(0o020000), S_ISBLK(0o060000), S_ISSOCK(0o140000)"""
import stat

# Each predicate is True for its own file-type mode.
assert stat.S_ISDIR(0o040755) is True, "S_ISDIR(dir)"
assert stat.S_ISREG(0o100644) is True, "S_ISREG(reg)"
assert stat.S_ISLNK(0o120755) is True, "S_ISLNK(lnk)"
assert stat.S_ISFIFO(0o010000) is True, "S_ISFIFO(fifo)"
assert stat.S_ISCHR(0o020000) is True, "S_ISCHR(chr)"
assert stat.S_ISBLK(0o060000) is True, "S_ISBLK(blk)"
assert stat.S_ISSOCK(0o140000) is True, "S_ISSOCK(sock)"

# A regular-file mode is rejected by the non-regular predicates.
reg = 0o100644
assert stat.S_ISDIR(reg) is False, "S_ISDIR(reg)"
assert stat.S_ISLNK(reg) is False, "S_ISLNK(reg)"
assert stat.S_ISFIFO(reg) is False, "S_ISFIFO(reg)"
assert stat.S_ISCHR(reg) is False, "S_ISCHR(reg)"
assert stat.S_ISBLK(reg) is False, "S_ISBLK(reg)"
assert stat.S_ISSOCK(reg) is False, "S_ISSOCK(reg)"

print("s_is_predicates_match_file_type OK")
"###);
    assert_output(&out, r###"s_is_predicates_match_file_type OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/stat/st_field_indices.py`.
#[test]
fn test_gen_behavior_std_libs_stat_st_field_indices() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "stat"
# dimension = "behavior"
# case = "st_field_indices"
# subject = "stat.ST_MODE"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_stat.py"
# status = "filled"
# ///
"""stat.ST_MODE: stat-result field index constants carry the documented positions: ST_MODE==0, ST_SIZE==6, ST_MTIME==8"""
import stat

# Field-index constants index into an os.stat_result tuple.
assert stat.ST_MODE == 0, "ST_MODE"
assert stat.ST_SIZE == 6, "ST_SIZE"
assert stat.ST_MTIME == 8, "ST_MTIME"

print("st_field_indices OK")
"###);
    assert_output(&out, r###"st_field_indices OK
"###);
}
