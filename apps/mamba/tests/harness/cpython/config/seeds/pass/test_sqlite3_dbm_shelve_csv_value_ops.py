# Atomic 281 pass conformance — sqlite3 module (hasattr connect/
# PARSE_DECLTYPES/PARSE_COLNAMES) + dbm module (hasattr open/whichdb/
# error) + shelve module (hasattr open/Shelf/BsdDbShelf/
# DbfilenameShelf) + csv module (hasattr reader/writer/DictReader/
# DictWriter/Dialect/Error/QUOTE_ALL/QUOTE_MINIMAL/QUOTE_NONE/
# QUOTE_NONNUMERIC/register_dialect/unregister_dialect/get_dialect/
# list_dialects/field_size_limit + QUOTE_MINIMAL/ALL/NONE/NONNUMERIC
# value contracts 0/1/2/3 + 'excel'/'excel-tab'/'unix' in
# list_dialects).
# All asserts match between CPython 3.12 and mamba.
import sqlite3
import dbm
import shelve
import csv


_ledger: list[int] = []

# 1) sqlite3 — hasattr top-level entries
assert hasattr(sqlite3, "connect") == True; _ledger.append(1)
assert hasattr(sqlite3, "PARSE_DECLTYPES") == True; _ledger.append(1)
assert hasattr(sqlite3, "PARSE_COLNAMES") == True; _ledger.append(1)

# 2) dbm — hasattr top-level entries
assert hasattr(dbm, "open") == True; _ledger.append(1)
assert hasattr(dbm, "whichdb") == True; _ledger.append(1)
assert hasattr(dbm, "error") == True; _ledger.append(1)

# 3) shelve — hasattr top-level entries
assert hasattr(shelve, "open") == True; _ledger.append(1)
assert hasattr(shelve, "Shelf") == True; _ledger.append(1)
assert hasattr(shelve, "BsdDbShelf") == True; _ledger.append(1)
assert hasattr(shelve, "DbfilenameShelf") == True; _ledger.append(1)

# 4) csv — hasattr reader/writer surface
assert hasattr(csv, "reader") == True; _ledger.append(1)
assert hasattr(csv, "writer") == True; _ledger.append(1)
assert hasattr(csv, "DictReader") == True; _ledger.append(1)
assert hasattr(csv, "DictWriter") == True; _ledger.append(1)

# 5) csv — hasattr dialect surface
assert hasattr(csv, "Dialect") == True; _ledger.append(1)
assert hasattr(csv, "Error") == True; _ledger.append(1)
assert hasattr(csv, "register_dialect") == True; _ledger.append(1)
assert hasattr(csv, "unregister_dialect") == True; _ledger.append(1)
assert hasattr(csv, "get_dialect") == True; _ledger.append(1)
assert hasattr(csv, "list_dialects") == True; _ledger.append(1)
assert hasattr(csv, "field_size_limit") == True; _ledger.append(1)

# 6) csv — hasattr quote-mode constants
assert hasattr(csv, "QUOTE_ALL") == True; _ledger.append(1)
assert hasattr(csv, "QUOTE_MINIMAL") == True; _ledger.append(1)
assert hasattr(csv, "QUOTE_NONE") == True; _ledger.append(1)
assert hasattr(csv, "QUOTE_NONNUMERIC") == True; _ledger.append(1)

# 7) csv — quote-mode value contracts
assert csv.QUOTE_MINIMAL == 0; _ledger.append(1)
assert csv.QUOTE_ALL == 1; _ledger.append(1)
assert csv.QUOTE_NONNUMERIC == 2; _ledger.append(1)
assert csv.QUOTE_NONE == 3; _ledger.append(1)

# 8) csv — built-in dialect membership
_dialects = csv.list_dialects()
assert ("excel" in _dialects) == True; _ledger.append(1)
assert ("excel-tab" in _dialects) == True; _ledger.append(1)
assert ("unix" in _dialects) == True; _ledger.append(1)

# 9) csv — built-in dialect count
assert len(_dialects) == 3; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_sqlite3_dbm_shelve_csv_value_ops {sum(_ledger)} asserts")
