# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_sqlite3_dbm_shelve_csv_silent"
# subject = "cpython321.lang_sqlite3_dbm_shelve_csv_silent"
# kind = "semantic"
# xfail = "CPython 3.12 seed spec; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/spec/lang_sqlite3_dbm_shelve_csv_silent.py"
# status = "filled"
# ///
"""cpython321.lang_sqlite3_dbm_shelve_csv_silent: execute CPython 3.12 seed lang_sqlite3_dbm_shelve_csv_silent"""
# mamba-xfail: CPython 3.12 seed spec; mamba promotion pending
# Operational AssertionPass divergence-spec fixture for the silent
# value-contract divergences of `hasattr(sqlite3, 'Connection')` (the
# documented "sqlite3 exposes the Connection class" — mamba returns
# False — sqlite3 module is a dict), `hasattr(sqlite3, 'Cursor')`
# (the documented "sqlite3 exposes the Cursor class" — mamba returns
# False), `hasattr(sqlite3, 'Error')` (the documented "sqlite3
# exposes the DB-API root Error exception" — mamba returns False),
# `hasattr(sqlite3, 'OperationalError')` (the documented "sqlite3
# exposes the OperationalError exception" — mamba returns False),
# `hasattr(sqlite3, 'register_adapter')` (the documented "sqlite3
# exposes the register_adapter type-mapper" — mamba returns False),
# `type(sqlite3.connect(':memory:')).__name__` (the documented
# "sqlite3.connect returns a Connection instance" — mamba returns
# 'dict' — connection is a dict handle), `hasattr(csv, 'Sniffer')`
# (the documented "csv exposes the Sniffer dialect-detector" —
# mamba returns False), `type(csv.Error).__name__` (the documented
# "csv.Error metatype is 'type'" — mamba returns 'csv.Error' — the
# class object reports its own qualified name as the metatype),
# `csv.list_dialects() == ['excel', 'excel-tab', 'unix']` (the
# documented "list_dialects returns alphabetic order" — mamba
# returns ['excel-tab', 'excel', 'unix'] — different ordering),
# and `hasattr(csv.writer(io.StringIO()), 'writerow')` (the
# documented "csv.writer returns a writer with writerow method" —
# mamba returns False — csv.writer returns a str).
# Ten-pack pinned to atomic 281.
#
# Behavioral edges that CONFORM on mamba (sqlite3 — hasattr connect/
# PARSE_DECLTYPES/PARSE_COLNAMES. dbm — hasattr open/whichdb/error.
# shelve — hasattr open/Shelf/BsdDbShelf/DbfilenameShelf. csv —
# hasattr reader/writer/DictReader/DictWriter/Dialect/Error/
# register_dialect/unregister_dialect/get_dialect/list_dialects/
# field_size_limit/QUOTE_ALL/QUOTE_MINIMAL/QUOTE_NONE/
# QUOTE_NONNUMERIC + value contracts 0/1/2/3 + 'excel'/'excel-tab'/
# 'unix' in list_dialects + len 3) are covered in the matching pass
# fixture `test_sqlite3_dbm_shelve_csv_value_ops`.
import sqlite3
import csv
import io


_ledger: list[int] = []

# 1) hasattr(sqlite3, 'Connection') — Connection class
#    (mamba: returns False — sqlite3 is a dict)
assert hasattr(sqlite3, "Connection") == True; _ledger.append(1)

# 2) hasattr(sqlite3, 'Cursor') — Cursor class
#    (mamba: returns False)
assert hasattr(sqlite3, "Cursor") == True; _ledger.append(1)

# 3) hasattr(sqlite3, 'Error') — DB-API root exception
#    (mamba: returns False)
assert hasattr(sqlite3, "Error") == True; _ledger.append(1)

# 4) hasattr(sqlite3, 'OperationalError') — OperationalError
#    (mamba: returns False)
assert hasattr(sqlite3, "OperationalError") == True; _ledger.append(1)

# 5) hasattr(sqlite3, 'register_adapter') — type-mapper
#    (mamba: returns False)
assert hasattr(sqlite3, "register_adapter") == True; _ledger.append(1)

# 6) type(sqlite3.connect(':memory:')).__name__ == 'Connection'
#    (mamba: returns 'dict' — connection is a dict handle)
assert type(sqlite3.connect(":memory:")).__name__ == "Connection"; _ledger.append(1)

# 7) hasattr(csv, 'Sniffer') — Sniffer dialect-detector
#    (mamba: returns False)
assert hasattr(csv, "Sniffer") == True; _ledger.append(1)

# 8) type(csv.Error).__name__ == 'type' — Error metatype
#    (mamba: returns 'csv.Error' — qualified-name metatype)
assert type(csv.Error).__name__ == "type"; _ledger.append(1)

# 9) csv.list_dialects() == ['excel', 'excel-tab', 'unix']
#    (mamba: returns ['excel-tab', 'excel', 'unix'] — different order)
assert csv.list_dialects() == ["excel", "excel-tab", "unix"]; _ledger.append(1)

# 10) hasattr(csv.writer(io.StringIO()), 'writerow') — writer object
#     (mamba: returns False — csv.writer returns a str)
assert hasattr(csv.writer(io.StringIO()), "writerow") == True; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_sqlite3_dbm_shelve_csv_silent {sum(_ledger)} asserts")
