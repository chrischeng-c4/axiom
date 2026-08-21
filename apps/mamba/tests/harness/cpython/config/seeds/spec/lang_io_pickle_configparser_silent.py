# Operational AssertionPass seed for SILENT divergences across
# the `io` extended class identifier surface + `io.BytesIO`
# instance class identity contract + `pickle.DEFAULT_PROTOCOL`
# integer-value contract + `csv.Sniffer` class identifier
# surface + `configparser` extended class / exception
# identifier surface + `configparser.ConfigParser` instance
# class identity contract pinned by atomic 194: `io` (the
# documented `open` / `IOBase` / `RawIOBase` / `BufferedIOBase`
# / `TextIOBase` / `BufferedReader` / `BufferedWriter` /
# `BufferedRandom` / `TextIOWrapper` / `FileIO` /
# `DEFAULT_BUFFER_SIZE` / `SEEK_SET` / `SEEK_CUR` / `SEEK_END`
# / `UnsupportedOperation` extended function / class /
# constant / exception identifier surface), `io.BytesIO` (the
# documented `BytesIO` class identity — `type(io.BytesIO(b"x"))
# .__name__ == "BytesIO"` on CPython; mamba collapses to a
# `dict` placeholder), `pickle` (the documented
# `DEFAULT_PROTOCOL == 4` integer-value contract — mamba
# returns 5), `csv` (the documented `Sniffer` class
# identifier — present on CPython; mamba elides), and
# `configparser` (the documented `RawConfigParser` /
# `Error` / `NoSectionError` / `NoOptionError` /
# `DuplicateOptionError` / `DuplicateSectionError` /
# `InterpolationError` / `ParsingError` /
# `BasicInterpolation` / `ExtendedInterpolation` /
# `DEFAULTSECT` / `MAX_INTERPOLATION_DEPTH` extended class /
# exception / constant identifier surface + the documented
# `ConfigParser` class identity — `type(configparser
# .ConfigParser()).__name__ == "ConfigParser"` on CPython;
# mamba collapses to a `dict` placeholder).
#
# The matching subset (partial io hasattr — BytesIO/StringIO
# only, full pickle hasattr + bytes-returning dumps + round-
# trip + HIGHEST_PROTOCOL == 5, full csv hasattr minus
# Sniffer + QUOTE_* integer values, partial configparser
# hasattr — ConfigParser only, full shutil hasattr +
# terminal_size return-type) is covered by
# `test_io_pickle_csv_shutil_value_ops`; this fixture pins
# the CPython-only contracts that mamba currently elides.
#
# Surface (CPython AssertionPass; mamba diverges silently):
#   • hasattr(io, "open") is True — documented function
#     identifier (mamba: False);
#   • hasattr(io, "IOBase") is True — documented class
#     identifier (mamba: False);
#   • hasattr(io, "RawIOBase") is True — documented class
#     identifier (mamba: False);
#   • hasattr(io, "BufferedIOBase") is True — documented
#     class identifier (mamba: False);
#   • hasattr(io, "TextIOBase") is True — documented class
#     identifier (mamba: False);
#   • hasattr(io, "BufferedReader") is True — documented
#     class identifier (mamba: False);
#   • hasattr(io, "BufferedWriter") is True — documented
#     class identifier (mamba: False);
#   • hasattr(io, "BufferedRandom") is True — documented
#     class identifier (mamba: False);
#   • hasattr(io, "TextIOWrapper") is True — documented
#     class identifier (mamba: False);
#   • hasattr(io, "FileIO") is True — documented class
#     identifier (mamba: False);
#   • hasattr(io, "DEFAULT_BUFFER_SIZE") is True —
#     documented constant identifier (mamba: False);
#   • hasattr(io, "SEEK_SET") is True — documented constant
#     identifier (mamba: False);
#   • hasattr(io, "SEEK_CUR") is True — documented constant
#     identifier (mamba: False);
#   • hasattr(io, "SEEK_END") is True — documented constant
#     identifier (mamba: False);
#   • hasattr(io, "UnsupportedOperation") is True —
#     documented exception identifier (mamba: False);
#   • type(io.BytesIO(b"x")).__name__ == "BytesIO" —
#     documented class identity (mamba: "dict");
#   • pickle.DEFAULT_PROTOCOL == 4 — documented integer-
#     value contract (mamba: 5);
#   • hasattr(csv, "Sniffer") is True — documented class
#     identifier (mamba: False);
#   • hasattr(configparser, "RawConfigParser") is True —
#     documented class identifier (mamba: False);
#   • hasattr(configparser, "Error") is True — documented
#     exception identifier (mamba: False);
#   • hasattr(configparser, "NoSectionError") is True —
#     documented exception identifier (mamba: False);
#   • hasattr(configparser, "NoOptionError") is True —
#     documented exception identifier (mamba: False);
#   • hasattr(configparser, "DuplicateOptionError") is
#     True — documented exception identifier (mamba:
#     False);
#   • hasattr(configparser, "DuplicateSectionError") is
#     True — documented exception identifier (mamba:
#     False);
#   • hasattr(configparser, "InterpolationError") is True
#     — documented exception identifier (mamba: False);
#   • hasattr(configparser, "ParsingError") is True —
#     documented exception identifier (mamba: False);
#   • hasattr(configparser, "BasicInterpolation") is True
#     — documented class identifier (mamba: False);
#   • hasattr(configparser, "ExtendedInterpolation") is
#     True — documented class identifier (mamba: False);
#   • hasattr(configparser, "DEFAULTSECT") is True —
#     documented constant identifier (mamba: False);
#   • hasattr(configparser, "MAX_INTERPOLATION_DEPTH") is
#     True — documented constant identifier (mamba: False);
#   • type(configparser.ConfigParser()).__name__ ==
#     "ConfigParser" — documented class identity (mamba:
#     "dict").
import io as _io_mod
import pickle as _pickle_mod
import csv as _csv_mod
import configparser as _configparser_mod
from typing import Any

# Module bindings retyped as `Any` to bypass Pyright stub-driven
# narrowing — every spec contract below probes documented public
# class / constant / exception identifier / instance-class
# identity behavior that mamba's bundled type stubs do not
# surface accurately.
io: Any = _io_mod
pickle: Any = _pickle_mod
csv: Any = _csv_mod
configparser: Any = _configparser_mod


_ledger: list[int] = []

# 1) io — extended function / class / constant / exception surface
assert hasattr(io, "open") == True; _ledger.append(1)
assert hasattr(io, "IOBase") == True; _ledger.append(1)
assert hasattr(io, "RawIOBase") == True; _ledger.append(1)
assert hasattr(io, "BufferedIOBase") == True; _ledger.append(1)
assert hasattr(io, "TextIOBase") == True; _ledger.append(1)
assert hasattr(io, "BufferedReader") == True; _ledger.append(1)
assert hasattr(io, "BufferedWriter") == True; _ledger.append(1)
assert hasattr(io, "BufferedRandom") == True; _ledger.append(1)
assert hasattr(io, "TextIOWrapper") == True; _ledger.append(1)
assert hasattr(io, "FileIO") == True; _ledger.append(1)
assert hasattr(io, "DEFAULT_BUFFER_SIZE") == True; _ledger.append(1)
assert hasattr(io, "SEEK_SET") == True; _ledger.append(1)
assert hasattr(io, "SEEK_CUR") == True; _ledger.append(1)
assert hasattr(io, "SEEK_END") == True; _ledger.append(1)
assert hasattr(io, "UnsupportedOperation") == True; _ledger.append(1)

# 2) io.BytesIO — instance class identity contract
assert type(io.BytesIO(b"x")).__name__ == "BytesIO"; _ledger.append(1)

# 3) pickle — DEFAULT_PROTOCOL integer-value contract
assert pickle.DEFAULT_PROTOCOL == 4; _ledger.append(1)

# 4) csv — extended class identifier surface
assert hasattr(csv, "Sniffer") == True; _ledger.append(1)

# 5) configparser — extended class / exception / constant surface
assert hasattr(configparser, "RawConfigParser") == True; _ledger.append(1)
assert hasattr(configparser, "Error") == True; _ledger.append(1)
assert hasattr(configparser, "NoSectionError") == True; _ledger.append(1)
assert hasattr(configparser, "NoOptionError") == True; _ledger.append(1)
assert hasattr(configparser, "DuplicateOptionError") == True; _ledger.append(1)
assert hasattr(configparser, "DuplicateSectionError") == True; _ledger.append(1)
assert hasattr(configparser, "InterpolationError") == True; _ledger.append(1)
assert hasattr(configparser, "ParsingError") == True; _ledger.append(1)
assert hasattr(configparser, "BasicInterpolation") == True; _ledger.append(1)
assert hasattr(configparser, "ExtendedInterpolation") == True; _ledger.append(1)
assert hasattr(configparser, "DEFAULTSECT") == True; _ledger.append(1)
assert hasattr(configparser, "MAX_INTERPOLATION_DEPTH") == True; _ledger.append(1)

# 6) configparser.ConfigParser — instance class identity contract
assert type(configparser.ConfigParser()).__name__ == "ConfigParser"; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_io_pickle_configparser_silent {sum(_ledger)} asserts")
