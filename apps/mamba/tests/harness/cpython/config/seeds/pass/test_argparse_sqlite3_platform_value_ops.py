# Atomic 315 pass conformance — argparse module (hasattr Argument
# Parser) + getopt module (hasattr getopt/gnu_getopt/GetoptError) +
# getpass module (hasattr getpass/getuser) + readline module (hasattr
# parse_and_bind/get_line_buffer/read_history_file/write_history_file/
# add_history/get_history_length/set_history_length) + shelve module
# (hasattr open/Shelf/BsdDbShelf/DbfilenameShelf) + dbm module (hasattr
# open/whichdb/error) + sqlite3 module (hasattr connect/PARSE_DECLTYPES
# /PARSE_COLNAMES) + fileinput module (hasattr input/FileInput/close/
# filename/lineno/filelineno/nextfile/isfirstline/isstdin) + platform
# module (hasattr system/platform/machine/processor/release/node/
# python_version + type str).
# All asserts match between CPython 3.12 and mamba.
import argparse
import getopt
import getpass
import readline
import shelve
import dbm
import sqlite3
import fileinput
import platform


_ledger: list[int] = []

# 1) argparse — hasattr (conformant subset)
assert hasattr(argparse, "ArgumentParser") == True; _ledger.append(1)

# 2) getopt — hasattr (conformant subset)
assert hasattr(getopt, "getopt") == True; _ledger.append(1)
assert hasattr(getopt, "gnu_getopt") == True; _ledger.append(1)
assert hasattr(getopt, "GetoptError") == True; _ledger.append(1)

# 3) getpass — hasattr (conformant subset)
assert hasattr(getpass, "getpass") == True; _ledger.append(1)
assert hasattr(getpass, "getuser") == True; _ledger.append(1)

# 4) readline — hasattr core surface
assert hasattr(readline, "parse_and_bind") == True; _ledger.append(1)
assert hasattr(readline, "get_line_buffer") == True; _ledger.append(1)
assert hasattr(readline, "read_history_file") == True; _ledger.append(1)
assert hasattr(readline, "write_history_file") == True; _ledger.append(1)
assert hasattr(readline, "add_history") == True; _ledger.append(1)
assert hasattr(readline, "get_history_length") == True; _ledger.append(1)
assert hasattr(readline, "set_history_length") == True; _ledger.append(1)

# 5) shelve — hasattr
assert hasattr(shelve, "open") == True; _ledger.append(1)
assert hasattr(shelve, "Shelf") == True; _ledger.append(1)
assert hasattr(shelve, "BsdDbShelf") == True; _ledger.append(1)
assert hasattr(shelve, "DbfilenameShelf") == True; _ledger.append(1)

# 6) dbm — hasattr
assert hasattr(dbm, "open") == True; _ledger.append(1)
assert hasattr(dbm, "whichdb") == True; _ledger.append(1)
assert hasattr(dbm, "error") == True; _ledger.append(1)

# 7) sqlite3 — hasattr (conformant subset)
assert hasattr(sqlite3, "connect") == True; _ledger.append(1)
assert hasattr(sqlite3, "PARSE_DECLTYPES") == True; _ledger.append(1)
assert hasattr(sqlite3, "PARSE_COLNAMES") == True; _ledger.append(1)

# 8) fileinput — hasattr (conformant subset)
assert hasattr(fileinput, "input") == True; _ledger.append(1)
assert hasattr(fileinput, "FileInput") == True; _ledger.append(1)
assert hasattr(fileinput, "close") == True; _ledger.append(1)
assert hasattr(fileinput, "filename") == True; _ledger.append(1)
assert hasattr(fileinput, "lineno") == True; _ledger.append(1)
assert hasattr(fileinput, "filelineno") == True; _ledger.append(1)
assert hasattr(fileinput, "nextfile") == True; _ledger.append(1)
assert hasattr(fileinput, "isfirstline") == True; _ledger.append(1)
assert hasattr(fileinput, "isstdin") == True; _ledger.append(1)

# 9) platform — hasattr (conformant subset)
assert hasattr(platform, "system") == True; _ledger.append(1)
assert hasattr(platform, "platform") == True; _ledger.append(1)
assert hasattr(platform, "machine") == True; _ledger.append(1)
assert hasattr(platform, "processor") == True; _ledger.append(1)
assert hasattr(platform, "release") == True; _ledger.append(1)
assert hasattr(platform, "node") == True; _ledger.append(1)
assert hasattr(platform, "python_version") == True; _ledger.append(1)

# 10) platform — type contracts (str returns)
assert type(platform.system()).__name__ == "str"; _ledger.append(1)
assert type(platform.python_version()).__name__ == "str"; _ledger.append(1)
assert type(platform.machine()).__name__ == "str"; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_argparse_sqlite3_platform_value_ops {sum(_ledger)} asserts")
