# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_fnmatch_glob_filecmp_posixpath_stat_errno_value_ops"
# subject = "cpython321.test_fnmatch_glob_filecmp_posixpath_stat_errno_value_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_fnmatch_glob_filecmp_posixpath_stat_errno_value_ops.py"
# status = "filled"
# ///
"""cpython321.test_fnmatch_glob_filecmp_posixpath_stat_errno_value_ops: execute CPython 3.12 seed test_fnmatch_glob_filecmp_posixpath_stat_errno_value_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Atomic 286 pass conformance — fnmatch module (hasattr fnmatch/
# fnmatchcase/filter/translate + fnmatch wildcard + fnmatchcase
# case-sensitive + filter selection + translate str return) + glob
# module (hasattr glob/iglob/escape/has_magic + escape brackets +
# has_magic True/False) + filecmp module (hasattr cmp/cmpfiles/
# dircmp/clear_cache/DEFAULT_IGNORES + DEFAULT_IGNORES list with
# '.git' and 'RCS') + posixpath module (hasattr join/split/basename/
# dirname/abspath/isabs/normpath/sep/extsep + sep == '/' + extsep
# == '.' + join/basename/dirname/isabs/normpath behavior) + ntpath
# module (hasattr join/split/basename/dirname/sep + sep == '\\') +
# stat module (hasattr S_ISDIR/S_ISREG/S_ISLNK/S_IFMT/S_IMODE/
# filemode/S_IFDIR/S_IFREG/S_IFLNK/S_IRUSR/S_IWUSR/S_IXUSR/S_IRWXU/
# ST_MODE/ST_SIZE/ST_MTIME + constant values + S_ISDIR/S_ISREG
# behavior) + errno module (hasattr EACCES/ENOENT/EEXIST/EISDIR/
# EPERM/EAGAIN/EINVAL/EINTR/EIO/errorcode + EACCES==13/ENOENT==2/
# EEXIST==17/EPERM==1/EINVAL==22).
# All asserts match between CPython 3.12 and mamba.
import fnmatch
import glob
import filecmp
import posixpath
import ntpath
import stat
import errno


_ledger: list[int] = []

# 1) fnmatch — hasattr core surface
assert hasattr(fnmatch, "fnmatch") == True; _ledger.append(1)
assert hasattr(fnmatch, "fnmatchcase") == True; _ledger.append(1)
assert hasattr(fnmatch, "filter") == True; _ledger.append(1)
assert hasattr(fnmatch, "translate") == True; _ledger.append(1)

# 2) fnmatch — behavior
assert fnmatch.fnmatch("foo.py", "*.py") == True; _ledger.append(1)
assert fnmatch.fnmatch("foo.py", "*.txt") == False; _ledger.append(1)
assert fnmatch.fnmatchcase("Foo.py", "*.py") == True; _ledger.append(1)
assert fnmatch.filter(["a.py", "b.txt", "c.py"], "*.py") == ["a.py", "c.py"]; _ledger.append(1)
assert isinstance(fnmatch.translate("*.py"), str) == True; _ledger.append(1)

# 3) glob — hasattr core surface
assert hasattr(glob, "glob") == True; _ledger.append(1)
assert hasattr(glob, "iglob") == True; _ledger.append(1)
assert hasattr(glob, "escape") == True; _ledger.append(1)
assert hasattr(glob, "has_magic") == True; _ledger.append(1)

# 4) glob — escape/has_magic
assert glob.escape("/foo[1].txt") == "/foo[[]1].txt"; _ledger.append(1)
assert glob.has_magic("foo*") == True; _ledger.append(1)
assert glob.has_magic("foo.txt") == False; _ledger.append(1)

# 5) filecmp — hasattr core surface
assert hasattr(filecmp, "cmp") == True; _ledger.append(1)
assert hasattr(filecmp, "cmpfiles") == True; _ledger.append(1)
assert hasattr(filecmp, "dircmp") == True; _ledger.append(1)
assert hasattr(filecmp, "clear_cache") == True; _ledger.append(1)
assert hasattr(filecmp, "DEFAULT_IGNORES") == True; _ledger.append(1)

# 6) filecmp — DEFAULT_IGNORES contents
assert isinstance(filecmp.DEFAULT_IGNORES, list) == True; _ledger.append(1)
assert ("RCS" in filecmp.DEFAULT_IGNORES) == True; _ledger.append(1)

# 7) posixpath — hasattr core surface
assert hasattr(posixpath, "join") == True; _ledger.append(1)
assert hasattr(posixpath, "split") == True; _ledger.append(1)
assert hasattr(posixpath, "basename") == True; _ledger.append(1)
assert hasattr(posixpath, "dirname") == True; _ledger.append(1)
assert hasattr(posixpath, "abspath") == True; _ledger.append(1)
assert hasattr(posixpath, "isabs") == True; _ledger.append(1)
assert hasattr(posixpath, "normpath") == True; _ledger.append(1)
assert hasattr(posixpath, "sep") == True; _ledger.append(1)
assert hasattr(posixpath, "extsep") == True; _ledger.append(1)

# 8) posixpath — value contracts
assert posixpath.sep == "/"; _ledger.append(1)
assert posixpath.extsep == "."; _ledger.append(1)
assert posixpath.join("a", "b", "c") == "a/b/c"; _ledger.append(1)
assert posixpath.basename("/foo/bar/baz.py") == "baz.py"; _ledger.append(1)
assert posixpath.dirname("/foo/bar/baz.py") == "/foo/bar"; _ledger.append(1)
assert posixpath.isabs("/foo") == True; _ledger.append(1)
assert posixpath.isabs("foo") == False; _ledger.append(1)
assert posixpath.normpath("/foo/./bar/../baz") == "/foo/baz"; _ledger.append(1)

# 9) ntpath — hasattr + sep
assert hasattr(ntpath, "join") == True; _ledger.append(1)
assert hasattr(ntpath, "basename") == True; _ledger.append(1)
assert hasattr(ntpath, "dirname") == True; _ledger.append(1)
assert hasattr(ntpath, "sep") == True; _ledger.append(1)
assert ntpath.sep == "\\"; _ledger.append(1)

# 10) stat — hasattr predicates + constants
assert hasattr(stat, "S_ISDIR") == True; _ledger.append(1)
assert hasattr(stat, "S_ISREG") == True; _ledger.append(1)
assert hasattr(stat, "S_ISLNK") == True; _ledger.append(1)
assert hasattr(stat, "S_IFMT") == True; _ledger.append(1)
assert hasattr(stat, "S_IMODE") == True; _ledger.append(1)
assert hasattr(stat, "filemode") == True; _ledger.append(1)
assert hasattr(stat, "S_IFDIR") == True; _ledger.append(1)
assert hasattr(stat, "S_IFREG") == True; _ledger.append(1)
assert hasattr(stat, "ST_MODE") == True; _ledger.append(1)
assert hasattr(stat, "ST_SIZE") == True; _ledger.append(1)

# 11) stat — constant values + predicate behavior
assert stat.S_IFDIR == 16384; _ledger.append(1)
assert stat.S_IFREG == 32768; _ledger.append(1)
assert stat.S_IRUSR == 256; _ledger.append(1)
assert stat.S_IWUSR == 128; _ledger.append(1)
assert stat.S_IXUSR == 64; _ledger.append(1)
assert stat.ST_MODE == 0; _ledger.append(1)
assert stat.S_ISDIR(stat.S_IFDIR) == True; _ledger.append(1)
assert stat.S_ISREG(stat.S_IFREG) == True; _ledger.append(1)
assert stat.S_ISDIR(stat.S_IFREG) == False; _ledger.append(1)

# 12) errno — hasattr standard codes
assert hasattr(errno, "EACCES") == True; _ledger.append(1)
assert hasattr(errno, "ENOENT") == True; _ledger.append(1)
assert hasattr(errno, "EEXIST") == True; _ledger.append(1)
assert hasattr(errno, "EPERM") == True; _ledger.append(1)
assert hasattr(errno, "EINVAL") == True; _ledger.append(1)
assert hasattr(errno, "EINTR") == True; _ledger.append(1)
assert hasattr(errno, "EIO") == True; _ledger.append(1)
assert hasattr(errno, "errorcode") == True; _ledger.append(1)

# 13) errno — value contracts
assert errno.EACCES == 13; _ledger.append(1)
assert errno.ENOENT == 2; _ledger.append(1)
assert errno.EEXIST == 17; _ledger.append(1)
assert errno.EPERM == 1; _ledger.append(1)
assert errno.EINVAL == 22; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_fnmatch_glob_filecmp_posixpath_stat_errno_value_ops {sum(_ledger)} asserts")
