# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_argparse_sqlite3_platform_silent"
# subject = "cpython321.lang_argparse_sqlite3_platform_silent"
# kind = "semantic"
# xfail = "CPython 3.12 seed spec; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/spec/lang_argparse_sqlite3_platform_silent.py"
# status = "filled"
# ///
"""cpython321.lang_argparse_sqlite3_platform_silent: execute CPython 3.12 seed lang_argparse_sqlite3_platform_silent"""
# mamba-xfail: CPython 3.12 seed spec; mamba promotion pending
# Operational AssertionPass divergence-spec fixture for the silent
# value-contract divergences of `hasattr(argparse, 'Namespace')` (the
# documented "argparse exposes the Namespace container class" — mamba
# returns False), `hasattr(argparse, 'FileType')` (the documented
# "argparse exposes the FileType action factory" — mamba returns False
# ), `hasattr(argparse, 'SUPPRESS')` (the documented "argparse exposes
# the SUPPRESS sentinel constant" — mamba returns False), `type(
# argparse.ArgumentParser()).__name__ == 'ArgumentParser'` (the
# documented "ArgumentParser() instantiates an ArgumentParser
# instance" — mamba returns 'dict' — constructor degrades to plain
# dict), `hasattr(sqlite3, 'Connection')` (the documented "sqlite3
# exposes the Connection class" — mamba returns False), `hasattr(
# sqlite3, 'IntegrityError')` (the documented "sqlite3 exposes the
# IntegrityError exception" — mamba returns False), `hasattr(sqlite3,
# 'register_adapter')` (the documented "sqlite3 exposes the register_
# adapter helper" — mamba returns False), `hasattr(getopt, 'error')`
# (the documented "getopt exposes the error alias for GetoptError" —
# mamba returns False), `hasattr(platform, 'uname')` (the documented
# "platform exposes the uname helper" — mamba returns False), and
# `hasattr(platform, 'python_implementation')` (the documented
# "platform exposes the python_implementation helper" — mamba returns
# False).
# Ten-pack pinned to atomic 315.
#
# Behavioral edges that CONFORM on mamba (argparse — hasattr Argument
# Parser. getopt — hasattr getopt/gnu_getopt/GetoptError. getpass —
# hasattr getpass/getuser. readline — hasattr parse_and_bind/get_line
# _buffer/read_history_file/write_history_file/add_history/get_history
# _length/set_history_length. shelve — hasattr open/Shelf/BsdDbShelf/
# DbfilenameShelf. dbm — hasattr open/whichdb/error. sqlite3 — hasattr
# connect/PARSE_DECLTYPES/PARSE_COLNAMES. fileinput — hasattr input/
# FileInput/close/filename/lineno/filelineno/nextfile/isfirstline/
# isstdin. platform — hasattr system/platform/machine/processor/release
# /node/python_version + type str returns) are covered in the matching
# pass fixture `test_argparse_sqlite3_platform_value_ops`.
import argparse
import getopt
import sqlite3
import platform


_ledger: list[int] = []

# 1) hasattr(argparse, 'Namespace') — Namespace container class
#    (mamba: returns False)
assert hasattr(argparse, "Namespace") == True; _ledger.append(1)

# 2) hasattr(argparse, 'FileType') — FileType action factory
#    (mamba: returns False)
assert hasattr(argparse, "FileType") == True; _ledger.append(1)

# 3) hasattr(argparse, 'SUPPRESS') — SUPPRESS sentinel constant
#    (mamba: returns False)
assert hasattr(argparse, "SUPPRESS") == True; _ledger.append(1)

# 4) type(argparse.ArgumentParser()).__name__ == 'ArgumentParser' — ArgumentParser instance
#    (mamba: returns 'dict' — constructor degrades to plain dict)
assert type(argparse.ArgumentParser()).__name__ == "ArgumentParser"; _ledger.append(1)

# 5) hasattr(sqlite3, 'Connection') — Connection class
#    (mamba: returns False)
assert hasattr(sqlite3, "Connection") == True; _ledger.append(1)

# 6) hasattr(sqlite3, 'IntegrityError') — IntegrityError exception
#    (mamba: returns False)
assert hasattr(sqlite3, "IntegrityError") == True; _ledger.append(1)

# 7) hasattr(sqlite3, 'register_adapter') — register_adapter helper
#    (mamba: returns False)
assert hasattr(sqlite3, "register_adapter") == True; _ledger.append(1)

# 8) hasattr(getopt, 'error') — error alias for GetoptError
#    (mamba: returns False)
assert hasattr(getopt, "error") == True; _ledger.append(1)

# 9) hasattr(platform, 'uname') — uname helper
#    (mamba: returns False)
assert hasattr(platform, "uname") == True; _ledger.append(1)

# 10) hasattr(platform, 'python_implementation') — python_implementation helper
#     (mamba: returns False)
assert hasattr(platform, "python_implementation") == True; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_argparse_sqlite3_platform_silent {sum(_ledger)} asserts")
