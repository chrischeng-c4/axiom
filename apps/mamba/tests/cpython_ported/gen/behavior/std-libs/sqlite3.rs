use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/std-libs/sqlite3/commit_persists_rollback_discards.py`.
#[test]
fn test_gen_behavior_std_libs_sqlite3_commit_persists_rollback_discards() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sqlite3"
# dimension = "behavior"
# case = "commit_persists_rollback_discards"
# subject = "sqlite3.Connection"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sqlite3.Connection: a committed INSERT persists while a subsequent uncommitted INSERT is discarded by rollback(); the surviving row count is 1"""
import sqlite3

conn = sqlite3.connect(":memory:")
cur = conn.cursor()
cur.execute("CREATE TABLE t1 (val INTEGER)")
cur.execute("INSERT INTO t1 VALUES (1)")
conn.commit()
cur.execute("INSERT INTO t1 VALUES (2)")
conn.rollback()
cur.execute("SELECT COUNT(*) FROM t1")
cnt = cur.fetchone()[0]
assert cnt == 1, f"rollback discards: count = {cnt!r}"
conn.close()

print("commit_persists_rollback_discards OK")
"###);
    assert_output(&out, r###"commit_persists_rollback_discards OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/sqlite3/context_manager_commits_on_success.py`.
#[test]
fn test_gen_behavior_std_libs_sqlite3_context_manager_commits_on_success() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sqlite3"
# dimension = "behavior"
# case = "context_manager_commits_on_success"
# subject = "sqlite3.Connection"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sqlite3.Connection: using the connection as a context manager auto-commits the enclosed writes when the block exits normally"""
import sqlite3

conn = sqlite3.connect(":memory:")
conn.execute("CREATE TABLE ctx (val INTEGER)")
with conn:
    conn.execute("INSERT INTO ctx VALUES (100)")
cur = conn.cursor()
cur.execute("SELECT COUNT(*) FROM ctx")
assert cur.fetchone()[0] == 1, "context manager commits the enclosed write"
conn.close()

print("context_manager_commits_on_success OK")
"###);
    assert_output(&out, r###"context_manager_commits_on_success OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/sqlite3/context_manager_rolls_back_on_exception.py`.
#[test]
fn test_gen_behavior_std_libs_sqlite3_context_manager_rolls_back_on_exception() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sqlite3"
# dimension = "behavior"
# case = "context_manager_rolls_back_on_exception"
# subject = "sqlite3.Connection"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sqlite3.Connection: when an exception escapes the connection context-manager block, the enclosed writes are rolled back and the exception re-raised"""
import sqlite3

conn = sqlite3.connect(":memory:")
conn.execute("CREATE TABLE ctx (val INTEGER)")
_raised = False
try:
    with conn:
        conn.execute("INSERT INTO ctx VALUES (200)")
        raise ValueError("force rollback")
except ValueError:
    _raised = True
assert _raised, "the escaping ValueError is re-raised"
cur = conn.cursor()
cur.execute("SELECT COUNT(*) FROM ctx")
assert cur.fetchone()[0] == 0, "the enclosed write was rolled back"
conn.close()

print("context_manager_rolls_back_on_exception OK")
"###);
    assert_output(&out, r###"context_manager_rolls_back_on_exception OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/sqlite3/cursor_iteration_yields_rows.py`.
#[test]
fn test_gen_behavior_std_libs_sqlite3_cursor_iteration_yields_rows() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sqlite3"
# dimension = "behavior"
# case = "cursor_iteration_yields_rows"
# subject = "sqlite3.Cursor"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sqlite3.Cursor: iterating a cursor after execute() yields each result row in order, one at a time"""
import sqlite3

conn = sqlite3.connect(":memory:")
conn.execute("CREATE TABLE iter_t (x INTEGER)")
conn.executemany("INSERT INTO iter_t VALUES (?)", [(i,) for i in range(5)])
conn.commit()
cur = conn.cursor()
cur.execute("SELECT x FROM iter_t ORDER BY x")
vals = [row[0] for row in cur]
assert vals == [0, 1, 2, 3, 4], f"cursor iter = {vals!r}"
conn.close()

print("cursor_iteration_yields_rows OK")
"###);
    assert_output(&out, r###"cursor_iteration_yields_rows OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/sqlite3/fetchmany_returns_at_most_size_rows.py`.
#[test]
fn test_gen_behavior_std_libs_sqlite3_fetchmany_returns_at_most_size_rows() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sqlite3"
# dimension = "behavior"
# case = "fetchmany_returns_at_most_size_rows"
# subject = "sqlite3.Cursor"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sqlite3.Cursor: fetchmany(size) returns at most `size` rows from the result set, in order"""
import sqlite3

conn = sqlite3.connect(":memory:")
conn.execute("CREATE TABLE many (n INTEGER)")
conn.executemany("INSERT INTO many VALUES (?)", [(i,) for i in range(10)])
conn.commit()
cur = conn.cursor()
cur.execute("SELECT n FROM many ORDER BY n")
chunk = cur.fetchmany(3)
assert len(chunk) == 3, f"fetchmany(3) = {len(chunk)!r}"
assert chunk[0][0] == 0, f"first = {chunk[0][0]!r}"
assert chunk[2][0] == 2, f"third = {chunk[2][0]!r}"
conn.close()

print("fetchmany_returns_at_most_size_rows OK")
"###);
    assert_output(&out, r###"fetchmany_returns_at_most_size_rows OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/sqlite3/lastrowid_reports_inserted_rowid.py`.
#[test]
fn test_gen_behavior_std_libs_sqlite3_lastrowid_reports_inserted_rowid() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sqlite3"
# dimension = "behavior"
# case = "lastrowid_reports_inserted_rowid"
# subject = "sqlite3.Cursor"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sqlite3.Cursor: cursor.lastrowid reports the rowid of the most recently inserted row"""
import sqlite3

conn = sqlite3.connect(":memory:")
cur = conn.cursor()
cur.execute("CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT)")
cur.execute("INSERT INTO users VALUES (3, 'Carol')")
assert cur.lastrowid == 3, f"lastrowid = {cur.lastrowid!r}"
conn.close()

print("lastrowid_reports_inserted_rowid OK")
"###);
    assert_output(&out, r###"lastrowid_reports_inserted_rowid OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/sqlite3/named_parameter_binding.py`.
#[test]
fn test_gen_behavior_std_libs_sqlite3_named_parameter_binding() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sqlite3"
# dimension = "behavior"
# case = "named_parameter_binding"
# subject = "sqlite3.Cursor"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sqlite3.Cursor: ':name' named placeholders bind a dict of values into INSERT/SELECT so the round-tripped value matches"""
import sqlite3

conn = sqlite3.connect(":memory:")
cur = conn.cursor()
cur.execute("CREATE TABLE items (name TEXT, value INTEGER)")
cur.execute("INSERT INTO items VALUES (:name, :value)", {"name": "named", "value": 99})
cur.execute("SELECT value FROM items WHERE name = :n", {"n": "named"})
v = cur.fetchone()[0]
assert v == 99, f"named parameter = {v!r}"
conn.close()

print("named_parameter_binding OK")
"###);
    assert_output(&out, r###"named_parameter_binding OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/sqlite3/null_roundtrips_as_none.py`.
#[test]
fn test_gen_behavior_std_libs_sqlite3_null_roundtrips_as_none() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sqlite3"
# dimension = "behavior"
# case = "null_roundtrips_as_none"
# subject = "sqlite3.Connection"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sqlite3.Connection: SQL NULL values are stored and retrieved as Python None for both TEXT and INTEGER columns"""
import sqlite3

conn = sqlite3.connect(":memory:")
conn.execute("CREATE TABLE nulls (a TEXT, b INTEGER)")
conn.execute("INSERT INTO nulls VALUES (NULL, NULL)")
conn.commit()
row = conn.execute("SELECT * FROM nulls").fetchone()
assert row[0] is None, f"NULL TEXT = {row[0]!r}"
assert row[1] is None, f"NULL INT = {row[1]!r}"
conn.close()

print("null_roundtrips_as_none OK")
"###);
    assert_output(&out, r###"null_roundtrips_as_none OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/sqlite3/qmark_parameter_binding.py`.
#[test]
fn test_gen_behavior_std_libs_sqlite3_qmark_parameter_binding() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sqlite3"
# dimension = "behavior"
# case = "qmark_parameter_binding"
# subject = "sqlite3.Cursor"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sqlite3.Cursor: '?' positional placeholders bind a tuple of values into INSERT/SELECT so the round-tripped value matches"""
import sqlite3

conn = sqlite3.connect(":memory:")
cur = conn.cursor()
cur.execute("CREATE TABLE items (name TEXT, value INTEGER)")
cur.execute("INSERT INTO items VALUES (?, ?)", ("test", 42))
cur.execute("SELECT value FROM items WHERE name = ?", ("test",))
v = cur.fetchone()[0]
assert v == 42, f"parameterized = {v!r}"
conn.close()

print("qmark_parameter_binding OK")
"###);
    assert_output(&out, r###"qmark_parameter_binding OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/sqlite3/row_factory_enables_named_access.py`.
#[test]
fn test_gen_behavior_std_libs_sqlite3_row_factory_enables_named_access() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sqlite3"
# dimension = "behavior"
# case = "row_factory_enables_named_access"
# subject = "sqlite3.Row"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sqlite3.Row: setting connection.row_factory = sqlite3.Row lets fetched rows be indexed by column name as well as by position"""
import sqlite3

conn = sqlite3.connect(":memory:")
conn.execute("CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT, age INTEGER)")
conn.execute("INSERT INTO users VALUES (1, 'Alice', 30)")
conn.commit()
conn.row_factory = sqlite3.Row
cur = conn.cursor()
cur.execute("SELECT * FROM users WHERE id = 1")
row = cur.fetchone()
assert row["name"] == "Alice", f"Row by name = {row['name']!r}"
assert row["age"] == 30, f"Row by name = {row['age']!r}"
assert row[1] == "Alice", f"Row by position = {row[1]!r}"
conn.close()

print("row_factory_enables_named_access OK")
"###);
    assert_output(&out, r###"row_factory_enables_named_access OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/sqlite3/rowcount_reports_affected_rows.py`.
#[test]
fn test_gen_behavior_std_libs_sqlite3_rowcount_reports_affected_rows() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sqlite3"
# dimension = "behavior"
# case = "rowcount_reports_affected_rows"
# subject = "sqlite3.Cursor"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sqlite3.Cursor: cursor.rowcount reports the number of rows affected by an UPDATE"""
import sqlite3

conn = sqlite3.connect(":memory:")
cur = conn.cursor()
cur.execute("CREATE TABLE users (id INTEGER PRIMARY KEY, age INTEGER)")
cur.execute("INSERT INTO users VALUES (1, 30)")
cur.execute("UPDATE users SET age = 31 WHERE id = 1")
assert cur.rowcount == 1, f"rowcount = {cur.rowcount!r}"
conn.close()

print("rowcount_reports_affected_rows OK")
"###);
    assert_output(&out, r###"rowcount_reports_affected_rows OK
"###);
}
