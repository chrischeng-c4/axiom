# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_argparse_configparser_csv_getopt_silent"
# subject = "cpython321.lang_argparse_configparser_csv_getopt_silent"
# kind = "semantic"
# xfail = "CPython 3.12 seed spec; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/spec/lang_argparse_configparser_csv_getopt_silent.py"
# status = "filled"
# ///
"""cpython321.lang_argparse_configparser_csv_getopt_silent: execute CPython 3.12 seed lang_argparse_configparser_csv_getopt_silent"""
# mamba-xfail: CPython 3.12 seed spec; mamba promotion pending
# Operational AssertionPass divergence-spec fixture for the silent
# value-contract divergences of `hasattr(argparse, 'Namespace')` (the
# documented "argparse exposes the Namespace class returned by
# parse_args()" — mamba returns False), `type(argparse.Argument
# Parser()).__name__` (the documented "ArgumentParser() returns an
# ArgumentParser instance" — mamba returns 'dict'), `hasattr
# (argparse, 'REMAINDER')` (the documented "argparse exposes the
# REMAINDER nargs sentinel" — mamba returns False), `hasattr
# (configparser, 'RawConfigParser')` (the documented "configparser
# exposes the RawConfigParser class" — mamba returns False),
# `configparser.DEFAULTSECT` (the documented "DEFAULTSECT is the
# string 'DEFAULT'" — mamba returns None), `type(configparser
# .ConfigParser()).__name__` (the documented "ConfigParser() returns
# a ConfigParser instance" — mamba returns 'dict'), `hasattr
# (configparser, 'NoSectionError')` (the documented "configparser
# exposes the NoSectionError exception class" — mamba returns
# False), `hasattr(csv, 'Sniffer')` (the documented "csv exposes
# the Sniffer dialect-detection helper" — mamba returns False),
# `hasattr(getopt, 'error')` (the documented "getopt exposes the
# `error` alias of GetoptError" — mamba returns False), and
# `csv.reader(['a;b'], delimiter=';')` (the documented "delimiter=
# kwarg changes the column separator — returns [['a', 'b']]" —
# mamba returns [['a;b']], ignoring the delimiter kwarg).
# Ten-pack pinned to atomic 267.
#
# Behavioral edges that CONFORM on mamba (argparse — hasattr
# ArgumentParser. configparser — hasattr ConfigParser. csv —
# hasattr reader/writer/DictReader/DictWriter/QUOTE_ALL/QUOTE_
# MINIMAL/QUOTE_NONE/Dialect/excel/excel_tab/unix_dialect/register_
# dialect/unregister_dialect/get_dialect/list_dialects + QUOTE_ALL
# ==1 / QUOTE_MINIMAL==0 / QUOTE_NONE==3 + reader value contracts +
# excel/excel_tab delimiter + list_dialects/get_dialect lookup.
# getopt — hasattr getopt/gnu_getopt/GetoptError + getopt parsing
# variants + gnu_getopt) are covered in the matching pass fixture
# `test_argparse_configparser_csv_getopt_value_ops`.
import argparse
import configparser
import csv
import getopt


_ledger: list[int] = []

# 1) hasattr(argparse, 'Namespace') — Namespace class
#    (mamba: returns False)
assert hasattr(argparse, "Namespace") == True; _ledger.append(1)

# 2) type(argparse.ArgumentParser()).__name__ == 'ArgumentParser'
#    (mamba: returns 'dict')
assert type(argparse.ArgumentParser()).__name__ == "ArgumentParser"; _ledger.append(1)

# 3) hasattr(argparse, 'REMAINDER') — nargs sentinel
#    (mamba: returns False)
assert hasattr(argparse, "REMAINDER") == True; _ledger.append(1)

# 4) hasattr(configparser, 'RawConfigParser') — RawConfigParser class
#    (mamba: returns False)
assert hasattr(configparser, "RawConfigParser") == True; _ledger.append(1)

# 5) configparser.DEFAULTSECT == 'DEFAULT'
#    (mamba: returns None)
assert configparser.DEFAULTSECT == "DEFAULT"; _ledger.append(1)

# 6) type(configparser.ConfigParser()).__name__ == 'ConfigParser'
#    (mamba: returns 'dict')
assert type(configparser.ConfigParser()).__name__ == "ConfigParser"; _ledger.append(1)

# 7) hasattr(configparser, 'NoSectionError') — exception class
#    (mamba: returns False)
assert hasattr(configparser, "NoSectionError") == True; _ledger.append(1)

# 8) hasattr(csv, 'Sniffer') — dialect-detection helper
#    (mamba: returns False)
assert hasattr(csv, "Sniffer") == True; _ledger.append(1)

# 9) hasattr(getopt, 'error') — alias of GetoptError
#    (mamba: returns False)
assert hasattr(getopt, "error") == True; _ledger.append(1)

# 10) csv.reader with delimiter=';' splits on semicolons
#     (mamba: returns [['a;b']] — delimiter kwarg ignored)
assert list(csv.reader(["a;b"], delimiter=";")) == [["a", "b"]]; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_argparse_configparser_csv_getopt_silent {sum(_ledger)} asserts")
