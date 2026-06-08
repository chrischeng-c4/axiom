# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_csv_configparser_silent"
# subject = "cpython321.lang_csv_configparser_silent"
# kind = "semantic"
# xfail = "CPython 3.12 seed spec; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/spec/lang_csv_configparser_silent.py"
# status = "filled"
# ///
"""cpython321.lang_csv_configparser_silent: execute CPython 3.12 seed lang_csv_configparser_silent"""
# mamba-xfail: CPython 3.12 seed spec; mamba promotion pending
# Operational AssertionPass seed for SILENT divergences across
# the `csv` extra class identifier surface +
# `csv.writer` instance class-identity contract +
# `configparser` module class / exception / sentinel
# identifier surface + `configparser.DEFAULTSECT`
# string-constant value contract + `configparser
# .ConfigParser` instance method identifier contract
# pinned by atomic 207: `csv` (the documented
# `Sniffer` class identifier — `Sniffer` + the
# documented `type(csv.writer(io.StringIO()))
# .__name__ == "writer"` class-identity contract —
# mamba collapses to "str"), and `configparser` (the
# documented class / interpolation / exception /
# sentinel identifier surface — `RawConfigParser` /
# `BasicInterpolation` / `ExtendedInterpolation` /
# `Interpolation` / `InterpolationError` /
# `InterpolationDepthError` /
# `InterpolationMissingOptionError` /
# `InterpolationSyntaxError` / `NoSectionError` /
# `NoOptionError` / `DuplicateSectionError` /
# `DuplicateOptionError` / `MissingSectionHeaderError`
# / `ParsingError` / `Error` / `DEFAULTSECT` /
# `MAX_INTERPOLATION_DEPTH` + the documented
# `configparser.DEFAULTSECT == "DEFAULT"` /
# `type(configparser.DEFAULTSECT).__name__ == "str"`
# string-constant value contract + the documented
# `configparser.ConfigParser().sections() == []`
# instance-method contract — mamba: instance collapses
# to dict).
#
# The matching subset (full calendar hasattr +
# leap-year/weekday/monthrange value contract, full
# fnmatch hasattr + glob-match/case-sensitive/
# filter-list/translation value contract, full glob
# hasattr + list-return/bracket-escape value
# contract, partial csv hasattr + integer-constant
# value contract, partial configparser hasattr) is
# covered by
# `test_calendar_fnmatch_glob_csv_value_ops`;
# this fixture pins the CPython-only contracts that
# mamba currently elides.
#
# Surface (CPython AssertionPass; mamba diverges silently):
#   • hasattr(csv, "Sniffer") is True — documented
#     class identifier (mamba: False);
#   • type(csv.writer(io.StringIO())).__name__ ==
#     "writer" — documented class-identity contract
#     (mamba: "str");
#   • hasattr(configparser, "RawConfigParser") is True
#     — documented class identifier (mamba: False);
#   • hasattr(configparser, "BasicInterpolation") is
#     True — documented class identifier (mamba: False);
#   • hasattr(configparser, "ExtendedInterpolation") is
#     True — documented class identifier (mamba: False);
#   • hasattr(configparser, "Interpolation") is True —
#     documented class identifier (mamba: False);
#   • hasattr(configparser, "InterpolationError") is True
#     — documented exception identifier (mamba: False);
#   • hasattr(configparser, "InterpolationDepthError") is
#     True — documented exception identifier (mamba: False);
#   • hasattr(configparser, "InterpolationMissingOptionError")
#     is True — documented exception identifier (mamba: False);
#   • hasattr(configparser, "InterpolationSyntaxError") is
#     True — documented exception identifier (mamba: False);
#   • hasattr(configparser, "NoSectionError") is True —
#     documented exception identifier (mamba: False);
#   • hasattr(configparser, "NoOptionError") is True —
#     documented exception identifier (mamba: False);
#   • hasattr(configparser, "DuplicateSectionError") is
#     True — documented exception identifier (mamba: False);
#   • hasattr(configparser, "DuplicateOptionError") is True
#     — documented exception identifier (mamba: False);
#   • hasattr(configparser, "MissingSectionHeaderError") is
#     True — documented exception identifier (mamba: False);
#   • hasattr(configparser, "ParsingError") is True —
#     documented exception identifier (mamba: False);
#   • hasattr(configparser, "Error") is True — documented
#     exception identifier (mamba: False);
#   • hasattr(configparser, "DEFAULTSECT") is True —
#     documented sentinel identifier (mamba: False);
#   • hasattr(configparser, "MAX_INTERPOLATION_DEPTH") is
#     True — documented sentinel identifier (mamba: False);
#   • configparser.DEFAULTSECT == "DEFAULT" — documented
#     string-constant value (mamba: None);
#   • type(configparser.DEFAULTSECT).__name__ == "str" —
#     documented constant-type contract (mamba: "NoneType");
#   • configparser.ConfigParser().sections() == [] —
#     documented instance-method contract (mamba:
#     AttributeError on dict).
import csv as _csv_mod
import configparser as _configparser_mod
import io as _io_mod
from typing import Any

# Module bindings retyped as `Any` to bypass Pyright stub-driven
# narrowing — every spec contract below probes documented public
# class identity / module-attribute / instance-method
# identifier behavior that mamba's bundled type stubs do not
# surface accurately.
csv: Any = _csv_mod
configparser: Any = _configparser_mod
io: Any = _io_mod


_ledger: list[int] = []

# 1) csv — extra class identifier surface
assert hasattr(csv, "Sniffer") == True; _ledger.append(1)

# 2) csv.writer — instance class-identity contract
_w = csv.writer(io.StringIO())
assert type(_w).__name__ == "writer"; _ledger.append(1)

# 3) configparser — class / interpolation / exception / sentinel identifier surface
assert hasattr(configparser, "RawConfigParser") == True; _ledger.append(1)
assert hasattr(configparser, "BasicInterpolation") == True; _ledger.append(1)
assert hasattr(configparser, "ExtendedInterpolation") == True; _ledger.append(1)
assert hasattr(configparser, "Interpolation") == True; _ledger.append(1)
assert hasattr(configparser, "InterpolationError") == True; _ledger.append(1)
assert hasattr(configparser, "InterpolationDepthError") == True; _ledger.append(1)
assert hasattr(configparser, "InterpolationMissingOptionError") == True; _ledger.append(1)
assert hasattr(configparser, "InterpolationSyntaxError") == True; _ledger.append(1)
assert hasattr(configparser, "NoSectionError") == True; _ledger.append(1)
assert hasattr(configparser, "NoOptionError") == True; _ledger.append(1)
assert hasattr(configparser, "DuplicateSectionError") == True; _ledger.append(1)
assert hasattr(configparser, "DuplicateOptionError") == True; _ledger.append(1)
assert hasattr(configparser, "MissingSectionHeaderError") == True; _ledger.append(1)
assert hasattr(configparser, "ParsingError") == True; _ledger.append(1)
assert hasattr(configparser, "Error") == True; _ledger.append(1)
assert hasattr(configparser, "DEFAULTSECT") == True; _ledger.append(1)
assert hasattr(configparser, "MAX_INTERPOLATION_DEPTH") == True; _ledger.append(1)

# 4) configparser.DEFAULTSECT — string-constant value contract
assert configparser.DEFAULTSECT == "DEFAULT"; _ledger.append(1)
assert type(configparser.DEFAULTSECT).__name__ == "str"; _ledger.append(1)

# 5) configparser.ConfigParser — instance-method contract
_cp = configparser.ConfigParser()
assert _cp.sections() == []; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_csv_configparser_silent {sum(_ledger)} asserts")
