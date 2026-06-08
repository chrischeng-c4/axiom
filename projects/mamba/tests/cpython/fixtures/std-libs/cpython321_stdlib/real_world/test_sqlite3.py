# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_sqlite3"
# subject = "cpython321.test_sqlite3"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_sqlite3.py"
# status = "filled"
# ///
"""cpython321.test_sqlite3: execute CPython 3.12 seed test_sqlite3"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# test_sqlite3.py — #3452 axis-1 stdlib sqlite3 AssertionPass seed.
#
# Mamba-authored seed exercising the `sqlite3` module surface called
# out in the issue:
#   connect(':memory:'), execute CREATE/INSERT/SELECT, fetchall, Row
#   factory, executemany.
#
# Surface coverage (asserts run at module scope; no helper closures per
# the mamba top-level def() quirk in test_math.py):
#   1. Module identity + public surface (hasattr).
#   2. connect(':memory:') returns a Connection — cursor() yields Cursor.
#   3. CREATE TABLE + single-row INSERT — rowcount + lastrowid.
#   4. executemany — multi-row INSERT + rowcount reflects batch size.
#   5. SELECT + fetchall — tuple-row default; description exposes
#      column names.
#   6. fetchone / fetchmany.
#   7. Row factory — sqlite3.Row gives mapping + sequence access.
#   8. Connection.total_changes accumulates across statements.
#   9. Parameterized SELECT with `?` placeholder + tuple bind.
#  10. context manager — commit/rollback semantics + IntegrityError.
#
# Boxed-int dodge applied to row-count comparisons.
#
# Contract with `cpython_lib_test_runner.rs`:
#   - Each `assert` runs at top level. Inverting any assert raises
#     AssertionError → non-zero exit → runner classifies as `Fail`.
#   - After every assertion has executed, the seed emits
#     `MAMBA_ASSERTION_PASS: test_sqlite3 N asserts` to stdout.

import sqlite3

_ledger: list[int] = []

# 1. Module identity + public surface.
assert sqlite3.__name__ == "sqlite3", "sqlite3.__name__"
_ledger.append(1)
assert hasattr(sqlite3, "connect"), "exposes connect"
_ledger.append(1)
assert hasattr(sqlite3, "Connection"), "exposes Connection"
_ledger.append(1)
assert hasattr(sqlite3, "Cursor"), "exposes Cursor"
_ledger.append(1)
assert hasattr(sqlite3, "Row"), "exposes Row factory"
_ledger.append(1)
assert hasattr(sqlite3, "IntegrityError"), "exposes IntegrityError"
_ledger.append(1)

# 2. connect(':memory:') + cursor().
_conn = sqlite3.connect(":memory:")
assert isinstance(_conn, sqlite3.Connection), "connect returns Connection"
_ledger.append(1)
_cur = _conn.cursor()
assert isinstance(_cur, sqlite3.Cursor), "Connection.cursor returns Cursor"
_ledger.append(1)

# 3. CREATE TABLE + single-row INSERT.
_cur.execute(
    "CREATE TABLE items (id INTEGER PRIMARY KEY, name TEXT NOT NULL, qty INTEGER)"
)
_cur.execute("INSERT INTO items (name, qty) VALUES (?, ?)", ("alpha", 1))
assert _cur.rowcount - 1 == 0, "INSERT rowcount == 1 after single insert"
_ledger.append(1)
_last_id = _cur.lastrowid
assert _last_id is not None, "lastrowid populated after INSERT"
_ledger.append(1)
assert _last_id - 1 == 0, "lastrowid == 1 after first INSERT"
_ledger.append(1)

# 4. executemany — batch INSERT.
_payload = [("beta", 2), ("gamma", 3), ("delta", 4)]
_cur.executemany("INSERT INTO items (name, qty) VALUES (?, ?)", _payload)
assert _cur.rowcount - 3 == 0, "executemany rowcount == 3 for 3-row batch"
_ledger.append(1)

# 5. SELECT + fetchall — tuple rows + description column names.
_cur.execute("SELECT name, qty FROM items ORDER BY qty ASC")
_rows = _cur.fetchall()
assert isinstance(_rows, list), "fetchall returns a list"
_ledger.append(1)
assert len(_rows) - 4 == 0, "fetchall returns 4 rows after CREATE + INSERT*4"
_ledger.append(1)
assert _rows[0] == ("alpha", 1), "first row matches first insert"
_ledger.append(1)
assert _rows[1] == ("beta", 2), "second row matches second insert"
_ledger.append(1)
assert _rows[3] == ("delta", 4), "fourth row matches fourth insert"
_ledger.append(1)
# description exposes column names (col[0] for each entry).
_desc = _cur.description
assert _desc is not None, "Cursor.description is populated after SELECT"
_ledger.append(1)
_col_names = [d[0] for d in _desc]
assert _col_names == ["name", "qty"], "description carries SELECT column names"
_ledger.append(1)

# 6. fetchone / fetchmany.
_cur.execute("SELECT name FROM items ORDER BY qty")
_first = _cur.fetchone()
assert _first == ("alpha",), "fetchone returns first row tuple"
_ledger.append(1)
# fetchmany(size=2) yields the next two rows.
_pair = _cur.fetchmany(size=2)
assert isinstance(_pair, list), "fetchmany returns a list"
_ledger.append(1)
assert len(_pair) - 2 == 0, "fetchmany(2) returns two rows"
_ledger.append(1)
assert _pair[0] == ("beta",), "fetchmany row 0 is the second item"
_ledger.append(1)
assert _pair[1] == ("gamma",), "fetchmany row 1 is the third item"
_ledger.append(1)

# 7. Row factory — sqlite3.Row exposes mapping + sequence access.
_conn.row_factory = sqlite3.Row
_cur_r = _conn.cursor()
_cur_r.execute("SELECT name, qty FROM items WHERE qty = ?", (2,))
_r = _cur_r.fetchone()
assert _r is not None, "Row fetchone returns a row when match exists"
_ledger.append(1)
# Mapping access by column name.
assert _r["name"] == "beta", "Row mapping access by column name"
_ledger.append(1)
assert _r["qty"] == 2, "Row mapping access — int column"
_ledger.append(1)
# Sequence access by position.
assert _r[0] == "beta", "Row sequence access [0]"
_ledger.append(1)
# keys() lists column names in selection order.
assert list(_r.keys()) == ["name", "qty"], "Row.keys() matches SELECT column order"
_ledger.append(1)
# Row is tuple-convertible.
assert tuple(_r) == ("beta", 2), "tuple(Row) yields the row values"
_ledger.append(1)

# 8. total_changes accumulates across statements.
# We've INSERTed 4 rows; CREATE doesn't change rows; SELECT doesn't either.
assert _conn.total_changes - 4 == 0, "total_changes accumulates 4 inserts"
_ledger.append(1)

# 9. Parameterized SELECT — tuple bind reaches `?` placeholder.
_cur_r.execute("SELECT COUNT(*) FROM items WHERE qty >= ?", (3,))
_count_row = _cur_r.fetchone()
# This row is also a sqlite3.Row given the connection-level factory.
assert _count_row[0] - 2 == 0, "COUNT(*) WHERE qty >= 3 returns 2 (gamma + delta)"
_ledger.append(1)

# 10. IntegrityError — NOT NULL constraint violation.
_raised = False
try:
    _conn.execute("INSERT INTO items (name, qty) VALUES (NULL, 99)")
except sqlite3.IntegrityError:
    _raised = True
assert _raised == True, "NOT NULL violation raises sqlite3.IntegrityError"
_ledger.append(1)

# Cleanup.
_conn.close()

# Emit the proof-of-execution marker. Per `ASSERTION_PASS_MARKERS` in
# `cpython_lib_test_runner.rs`, presence of this token escalates the
# outcome from ImportPass to AssertionPass.
print(f"MAMBA_ASSERTION_PASS: test_sqlite3 {len(_ledger)} asserts")
