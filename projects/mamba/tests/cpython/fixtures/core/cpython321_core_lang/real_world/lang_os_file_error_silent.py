# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_os_file_error_silent"
# subject = "cpython321.lang_os_file_error_silent"
# kind = "semantic"
# xfail = "CPython 3.12 seed spec; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/spec/lang_os_file_error_silent.py"
# status = "filled"
# ///
"""cpython321.lang_os_file_error_silent: execute CPython 3.12 seed lang_os_file_error_silent"""
# mamba-xfail: CPython 3.12 seed spec; mamba promotion pending
# Spec seed for CPython FileNotFoundError / FileExistsError /
# IsADirectoryError contract on the `os` / `os.path` / `open()`
# corners that mamba silently returns empty / zeroed / None values
# from. Surface: CPython raises (1) `FileNotFoundError` from
# `os.listdir` / `os.stat` / `os.remove` / `os.rmdir` / `os.rename`
# when the path doesn't exist — those are part of the OSError family
# tied to errno 2 (ENOENT) — not silent empty `[]` / zeroed dict /
# `None`; (2) `FileExistsError` from `os.mkdir` on an existing
# directory — errno 17 (EEXIST) — not silent `None`; (3)
# `IsADirectoryError` from reading a path that is a directory —
# errno 21 (EISDIR) — not silent empty `''`. Existing
# `lang_*_silent` seeds cover arithmetic / typing corners but no
# filesystem-error family yet.
#
# Probes (every form CPython rejects, mamba silently coerces):
#   • os.listdir(missing)          → mamba: []         (FileNotFoundError)
#   • os.stat(missing)             → mamba: zeroed-dict (FileNotFoundError)
#   • os.remove(missing)           → mamba: None       (FileNotFoundError)
#   • os.rmdir(missing)            → mamba: None       (FileNotFoundError)
#   • os.rename(missing, dst)      → mamba: None       (FileNotFoundError)
#   • os.mkdir(existing_dir)       → mamba: None       (FileExistsError)
#   • open(directory_path).read()  → mamba: ''         (IsADirectoryError)
#
# CPython contract:
#   os.listdir(missing)            → FileNotFoundError(errno=2,
#                                       "No such file or directory");
#   os.stat(missing)
#   os.remove(missing)
#   os.rmdir(missing)
#   os.rename(missing, dst)        → FileNotFoundError(errno=2);
#   os.mkdir(existing)             → FileExistsError(errno=17, "File
#                                       exists");
#   open("/tmp").read()            → IsADirectoryError(errno=21, "Is
#                                       a directory").
#
# FileNotFoundError / FileExistsError / IsADirectoryError are all
# subclasses of OSError, so each probe catches via the most specific
# subclass — establishing that the CPython hierarchy is preserved.
import os
_ledger: list[int] = []

_missing = "/__mamba_spec_missing_xyz_42__path__no__exist__"
_missing2 = "/__mamba_spec_missing_xyz_43__different__"
_existing_dir = "/tmp"

# os.listdir on missing path
try:
    _ = os.listdir(_missing)
    raise AssertionError("os.listdir(missing) must raise FileNotFoundError")
except FileNotFoundError:
    _ledger.append(1)

# os.stat on missing path
try:
    _ = os.stat(_missing)
    raise AssertionError("os.stat(missing) must raise FileNotFoundError")
except FileNotFoundError:
    _ledger.append(1)

# os.remove on missing path
try:
    _ = os.remove(_missing)
    raise AssertionError("os.remove(missing) must raise FileNotFoundError")
except FileNotFoundError:
    _ledger.append(1)

# os.rmdir on missing path
try:
    _ = os.rmdir(_missing)
    raise AssertionError("os.rmdir(missing) must raise FileNotFoundError")
except FileNotFoundError:
    _ledger.append(1)

# os.rename with missing source
try:
    _ = os.rename(_missing, _missing2)
    raise AssertionError("os.rename(missing, dst) must raise FileNotFoundError")
except FileNotFoundError:
    _ledger.append(1)

# os.mkdir on already-existing directory
try:
    _ = os.mkdir(_existing_dir)
    raise AssertionError("os.mkdir(existing_dir) must raise FileExistsError")
except FileExistsError:
    _ledger.append(1)

# open() then read() on a directory path
try:
    _f = open(_existing_dir, "r")
    _ = _f.read()
    raise AssertionError("open(dir).read() must raise IsADirectoryError")
except IsADirectoryError:
    _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_os_file_error_silent {sum(_ledger)} asserts")
