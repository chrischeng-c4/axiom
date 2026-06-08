# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_tempfile_glob_os_silent"
# subject = "cpython321.lang_tempfile_glob_os_silent"
# kind = "semantic"
# xfail = "CPython 3.12 seed spec; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/spec/lang_tempfile_glob_os_silent.py"
# status = "filled"
# ///
"""cpython321.lang_tempfile_glob_os_silent: execute CPython 3.12 seed lang_tempfile_glob_os_silent"""
# mamba-xfail: CPython 3.12 seed spec; mamba promotion pending
# Operational AssertionPass divergence-spec fixture for the silent
# value-contract divergences of `hasattr(tempfile, 'template')` (the
# documented "tempfile exposes the legacy template prefix" — mamba
# returns False), `hasattr(glob, 'fnmatch')` (the documented "glob
# re-exposes the fnmatch helper" — mamba returns False), `hasattr(os,
# 'chdir')` (the documented "os exposes the chdir directory-change
# call" — mamba returns False), `hasattr(os, 'unlink')` (the documented
# "os exposes the unlink alias for remove" — mamba returns False),
# `hasattr(os, 'putenv')` (the documented "os exposes the putenv
# environment helper" — mamba returns False), `hasattr(os, 'open')`
# (the documented "os exposes the low-level open file-descriptor
# call" — mamba returns False), `hasattr(os, 'close')` (the documented
# "os exposes the low-level close call" — mamba returns False),
# `hasattr(os, 'read')` (the documented "os exposes the low-level read
# call" — mamba returns False), `hasattr(os, 'lseek')` (the documented
# "os exposes the lseek call" — mamba returns False), and `hasattr(os,
# 'O_RDONLY')` (the documented "os exposes the O_RDONLY open-flag
# constant" — mamba returns False).
# Ten-pack pinned to atomic 317.
#
# Behavioral edges that CONFORM on mamba (tempfile — hasattr Temporary
# File/NamedTemporaryFile/SpooledTemporaryFile/TemporaryDirectory/
# mkstemp/mkdtemp/mktemp/gettempdir/gettempdirb/gettempprefix/get
# tempprefixb/tempdir + str return types. shutil — hasattr copy/copy2/
# copyfile/copytree/copyfileobj/copymode/copystat/rmtree/move/disk_
# usage/chown/which/ignore_patterns/Error/SameFileError/SpecialFile
# Error/ExecError/ReadError/RegistryError/make_archive/unpack_archive/
# register_archive_format/get_archive_formats/get_unpack_formats/COPY_
# BUFSIZE/get_terminal_size. glob — hasattr glob/iglob/escape/has_
# magic. fnmatch — hasattr fnmatch/fnmatchcase/filter/translate.
# pathlib — hasattr Path/PurePath/PurePosixPath/PureWindowsPath/Posix
# Path/WindowsPath. os — hasattr getcwd/listdir/mkdir/makedirs/rmdir/
# remove/rename/environ/getenv/sep/altsep/pathsep/linesep/extsep/name/
# devnull/curdir/pardir/stat/lstat/access/getpid/getppid/getuid/
# geteuid/getgid/getegid/cpu_count/urandom + sep=='/' + linesep=='\n'
# + pathsep==':' + name=='posix') are covered in the matching pass
# fixture `test_tempfile_shutil_pathlib_value_ops`.
import tempfile
import glob
import os


_ledger: list[int] = []

# 1) hasattr(tempfile, 'template') — legacy template prefix
#    (mamba: returns False)
assert hasattr(tempfile, "template") == True; _ledger.append(1)

# 2) hasattr(glob, 'fnmatch') — fnmatch re-export
#    (mamba: returns False)
assert hasattr(glob, "fnmatch") == True; _ledger.append(1)

# 3) hasattr(os, 'chdir') — chdir directory-change call
#    (mamba: returns False)
assert hasattr(os, "chdir") == True; _ledger.append(1)

# 4) hasattr(os, 'unlink') — unlink alias for remove
#    (mamba: returns False)
assert hasattr(os, "unlink") == True; _ledger.append(1)

# 5) hasattr(os, 'putenv') — putenv environment helper
#    (mamba: returns False)
assert hasattr(os, "putenv") == True; _ledger.append(1)

# 6) hasattr(os, 'open') — low-level open file-descriptor call
#    (mamba: returns False)
assert hasattr(os, "open") == True; _ledger.append(1)

# 7) hasattr(os, 'close') — low-level close call
#    (mamba: returns False)
assert hasattr(os, "close") == True; _ledger.append(1)

# 8) hasattr(os, 'read') — low-level read call
#    (mamba: returns False)
assert hasattr(os, "read") == True; _ledger.append(1)

# 9) hasattr(os, 'lseek') — lseek call
#    (mamba: returns False)
assert hasattr(os, "lseek") == True; _ledger.append(1)

# 10) hasattr(os, 'O_RDONLY') — O_RDONLY open-flag constant
#     (mamba: returns False)
assert hasattr(os, "O_RDONLY") == True; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_tempfile_glob_os_silent {sum(_ledger)} asserts")
