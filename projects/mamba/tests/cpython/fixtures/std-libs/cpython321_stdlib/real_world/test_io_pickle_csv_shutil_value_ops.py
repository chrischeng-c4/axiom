# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_io_pickle_csv_shutil_value_ops"
# subject = "cpython321.test_io_pickle_csv_shutil_value_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_io_pickle_csv_shutil_value_ops.py"
# status = "filled"
# ///
"""cpython321.test_io_pickle_csv_shutil_value_ops: execute CPython 3.12 seed test_io_pickle_csv_shutil_value_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for the value contract of the
# `io` / `pickle` / `csv` / `configparser` / `shutil` five-pack
# pinned to atomic 194: `io` (the documented partial module-level
# class identifier hasattr surface — `BytesIO` / `StringIO`; the
# extended `open` / `IOBase` / `RawIOBase` / `BufferedIOBase` /
# `TextIOBase` / `BufferedReader` / `BufferedWriter` /
# `BufferedRandom` / `TextIOWrapper` / `FileIO` /
# `DEFAULT_BUFFER_SIZE` / `SEEK_SET` / `SEEK_CUR` / `SEEK_END` /
# `UnsupportedOperation` surface DIVERGES on mamba and is moved
# to the spec fixture), `pickle` (the documented full module-
# level helper hasattr surface — `dumps` / `loads` / `dump` /
# `load` / `HIGHEST_PROTOCOL` / `Pickler` / `Unpickler` /
# `PickleError` / `PicklingError` / `UnpicklingError` /
# `PickleBuffer` + the documented bytes-returning dumps + the
# documented dumps/loads round-trip value contract + the
# documented HIGHEST_PROTOCOL == 5 integer-value contract; the
# `DEFAULT_PROTOCOL == 4` value contract DIVERGES on mamba —
# moved to spec fixture), `csv` (the documented full module-
# level helper hasattr surface minus `Sniffer` — `reader` /
# `writer` / `DictReader` / `DictWriter` / `QUOTE_ALL` /
# `QUOTE_MINIMAL` / `QUOTE_NONNUMERIC` / `QUOTE_NONE` /
# `Dialect` / `excel` / `excel_tab` / `unix_dialect` / `Error`
# / `field_size_limit` / `get_dialect` / `list_dialects` /
# `register_dialect` / `unregister_dialect` + the documented
# QUOTE_MINIMAL == 0 / QUOTE_ALL == 1 / QUOTE_NONNUMERIC == 2 /
# QUOTE_NONE == 3 integer-value contract), `configparser` (the
# documented partial module-level class identifier hasattr
# surface — only `ConfigParser`; the extended `RawConfigParser`
# / `Error` / `NoSectionError` / `NoOptionError` /
# `DuplicateOptionError` / `DuplicateSectionError` /
# `InterpolationError` / `ParsingError` / `BasicInterpolation`
# / `ExtendedInterpolation` / `DEFAULTSECT` /
# `MAX_INTERPOLATION_DEPTH` surface DIVERGES on mamba — moved
# to spec fixture), and `shutil` (the documented full module-
# level helper hasattr surface — `copy` / `copy2` / `copyfile`
# / `copyfileobj` / `copytree` / `copymode` / `copystat` /
# `move` / `rmtree` / `make_archive` / `unpack_archive` /
# `get_archive_formats` / `get_unpack_formats` /
# `register_archive_format` / `disk_usage` / `chown` / `which`
# / `get_terminal_size` / `ignore_patterns` / `Error` /
# `SameFileError` + the documented terminal_size named-tuple
# return-type contract).
#
# Behavioral edges that DIVERGE on mamba
# (hasattr(io, "open") / "IOBase" / "RawIOBase" /
# "BufferedIOBase" / "TextIOBase" / "BufferedReader" /
# "BufferedWriter" / "BufferedRandom" / "TextIOWrapper" /
# "FileIO" / "DEFAULT_BUFFER_SIZE" / "SEEK_SET" / "SEEK_CUR"
# / "SEEK_END" / "UnsupportedOperation" all False on mamba,
# type(io.BytesIO(b"x")).__name__ collapses to "dict" on mamba,
# pickle.DEFAULT_PROTOCOL == 4 on CPython but == 5 on mamba,
# hasattr(csv, "Sniffer") False on mamba,
# hasattr(configparser, "RawConfigParser") / "Error" /
# "NoSectionError" / "NoOptionError" / "DuplicateOptionError"
# / "DuplicateSectionError" / "InterpolationError" /
# "ParsingError" / "BasicInterpolation" /
# "ExtendedInterpolation" / "DEFAULTSECT" /
# "MAX_INTERPOLATION_DEPTH" all False on mamba,
# type(configparser.ConfigParser()).__name__ collapses to
# "dict" on mamba) are covered in the matching spec fixture
# `lang_io_pickle_configparser_silent`.
import io
import pickle
import csv
import configparser
import shutil


_ledger: list[int] = []

# 1) io — partial module-level class identifier hasattr surface
#    (open / IOBase / RawIOBase / BufferedIOBase / TextIOBase /
#    BufferedReader / BufferedWriter / BufferedRandom /
#    TextIOWrapper / FileIO / DEFAULT_BUFFER_SIZE / SEEK_SET /
#    SEEK_CUR / SEEK_END / UnsupportedOperation DIVERGE —
#    moved to spec fixture)
assert hasattr(io, "BytesIO") == True; _ledger.append(1)
assert hasattr(io, "StringIO") == True; _ledger.append(1)

# 2) pickle — full module-level helper hasattr surface
#    (DEFAULT_PROTOCOL value DIVERGES — moved to spec fixture)
assert hasattr(pickle, "dumps") == True; _ledger.append(1)
assert hasattr(pickle, "loads") == True; _ledger.append(1)
assert hasattr(pickle, "dump") == True; _ledger.append(1)
assert hasattr(pickle, "load") == True; _ledger.append(1)
assert hasattr(pickle, "HIGHEST_PROTOCOL") == True; _ledger.append(1)
assert hasattr(pickle, "DEFAULT_PROTOCOL") == True; _ledger.append(1)
assert hasattr(pickle, "Pickler") == True; _ledger.append(1)
assert hasattr(pickle, "Unpickler") == True; _ledger.append(1)
assert hasattr(pickle, "PickleError") == True; _ledger.append(1)
assert hasattr(pickle, "PicklingError") == True; _ledger.append(1)
assert hasattr(pickle, "UnpicklingError") == True; _ledger.append(1)
assert hasattr(pickle, "PickleBuffer") == True; _ledger.append(1)

# 3) pickle — bytes-returning dumps + round-trip + HIGHEST_PROTOCOL value
_payload = pickle.dumps([1, 2, 3])
assert type(_payload).__name__ == "bytes"; _ledger.append(1)
assert pickle.loads(_payload) == [1, 2, 3]; _ledger.append(1)
assert pickle.HIGHEST_PROTOCOL == 5; _ledger.append(1)

# 4) csv — full module hasattr surface minus Sniffer
#    (Sniffer DIVERGES — moved to spec fixture)
assert hasattr(csv, "reader") == True; _ledger.append(1)
assert hasattr(csv, "writer") == True; _ledger.append(1)
assert hasattr(csv, "DictReader") == True; _ledger.append(1)
assert hasattr(csv, "DictWriter") == True; _ledger.append(1)
assert hasattr(csv, "QUOTE_ALL") == True; _ledger.append(1)
assert hasattr(csv, "QUOTE_MINIMAL") == True; _ledger.append(1)
assert hasattr(csv, "QUOTE_NONNUMERIC") == True; _ledger.append(1)
assert hasattr(csv, "QUOTE_NONE") == True; _ledger.append(1)
assert hasattr(csv, "Dialect") == True; _ledger.append(1)
assert hasattr(csv, "excel") == True; _ledger.append(1)
assert hasattr(csv, "excel_tab") == True; _ledger.append(1)
assert hasattr(csv, "unix_dialect") == True; _ledger.append(1)
assert hasattr(csv, "Error") == True; _ledger.append(1)
assert hasattr(csv, "field_size_limit") == True; _ledger.append(1)
assert hasattr(csv, "get_dialect") == True; _ledger.append(1)
assert hasattr(csv, "list_dialects") == True; _ledger.append(1)
assert hasattr(csv, "register_dialect") == True; _ledger.append(1)
assert hasattr(csv, "unregister_dialect") == True; _ledger.append(1)

# 5) csv — integer-value contract
assert csv.QUOTE_MINIMAL == 0; _ledger.append(1)
assert csv.QUOTE_ALL == 1; _ledger.append(1)
assert csv.QUOTE_NONNUMERIC == 2; _ledger.append(1)
assert csv.QUOTE_NONE == 3; _ledger.append(1)

# 6) configparser — partial module-level class identifier hasattr
#    (RawConfigParser / Error / NoSectionError / NoOptionError /
#    DuplicateOptionError / DuplicateSectionError /
#    InterpolationError / ParsingError / BasicInterpolation /
#    ExtendedInterpolation / DEFAULTSECT /
#    MAX_INTERPOLATION_DEPTH DIVERGE — moved to spec fixture)
assert hasattr(configparser, "ConfigParser") == True; _ledger.append(1)

# 7) shutil — full module hasattr surface
assert hasattr(shutil, "copy") == True; _ledger.append(1)
assert hasattr(shutil, "copy2") == True; _ledger.append(1)
assert hasattr(shutil, "copyfile") == True; _ledger.append(1)
assert hasattr(shutil, "copyfileobj") == True; _ledger.append(1)
assert hasattr(shutil, "copytree") == True; _ledger.append(1)
assert hasattr(shutil, "copymode") == True; _ledger.append(1)
assert hasattr(shutil, "copystat") == True; _ledger.append(1)
assert hasattr(shutil, "move") == True; _ledger.append(1)
assert hasattr(shutil, "rmtree") == True; _ledger.append(1)
assert hasattr(shutil, "make_archive") == True; _ledger.append(1)
assert hasattr(shutil, "unpack_archive") == True; _ledger.append(1)
assert hasattr(shutil, "get_archive_formats") == True; _ledger.append(1)
assert hasattr(shutil, "get_unpack_formats") == True; _ledger.append(1)
assert hasattr(shutil, "register_archive_format") == True; _ledger.append(1)
assert hasattr(shutil, "disk_usage") == True; _ledger.append(1)
assert hasattr(shutil, "chown") == True; _ledger.append(1)
assert hasattr(shutil, "which") == True; _ledger.append(1)
assert hasattr(shutil, "get_terminal_size") == True; _ledger.append(1)
assert hasattr(shutil, "ignore_patterns") == True; _ledger.append(1)
assert hasattr(shutil, "Error") == True; _ledger.append(1)
assert hasattr(shutil, "SameFileError") == True; _ledger.append(1)

# 8) shutil — terminal_size named-tuple return-type contract
assert type(shutil.get_terminal_size()).__name__ == "terminal_size"; _ledger.append(1)

# NB: hasattr(io, "open") / "IOBase" / "RawIOBase" /
# "BufferedIOBase" / "TextIOBase" / "BufferedReader" /
# "BufferedWriter" / "BufferedRandom" / "TextIOWrapper" /
# "FileIO" / "DEFAULT_BUFFER_SIZE" / "SEEK_SET" / "SEEK_CUR" /
# "SEEK_END" / "UnsupportedOperation" all False on mamba,
# type(io.BytesIO(b"x")).__name__ collapses to "dict" on mamba,
# pickle.DEFAULT_PROTOCOL == 4 on CPython but == 5 on mamba,
# hasattr(csv, "Sniffer") False on mamba,
# hasattr(configparser, "RawConfigParser") / "Error" /
# "NoSectionError" / "NoOptionError" / "DuplicateOptionError"
# / "DuplicateSectionError" / "InterpolationError" /
# "ParsingError" / "BasicInterpolation" /
# "ExtendedInterpolation" / "DEFAULTSECT" /
# "MAX_INTERPOLATION_DEPTH" all False on mamba,
# type(configparser.ConfigParser()).__name__ collapses to
# "dict" on mamba — all DIVERGE on mamba — moved to the
# divergence-spec fixture.

print(f"MAMBA_ASSERTION_PASS: test_io_pickle_csv_shutil_value_ops {sum(_ledger)} asserts")
