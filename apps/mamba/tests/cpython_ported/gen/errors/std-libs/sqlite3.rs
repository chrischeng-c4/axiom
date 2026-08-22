use super::super::super::super::harness::*;

/// Ported from `tests/cpython/errors/std-libs/sqlite3/bad_sql_raises.py`.
#[test]
fn test_gen_errors_std_libs_sqlite3_bad_sql_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sqlite3"
# dimension = "errors"
# case = "bad_sql_raises"
# subject = "sqlite3.Connection"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sqlite3.Connection: bad_sql_raises (errors)."""
import sqlite3

_raised = False
try:
    sqlite3.connect(":memory:").execute("NOT VALID SQL")
except sqlite3.OperationalError:
    _raised = True
assert _raised, "bad_sql_raises: expected sqlite3.OperationalError"
print("bad_sql_raises OK")
"###);
    assert_output(&out, r###"bad_sql_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/sqlite3/connect_directory_raises.py`.
#[test]
fn test_gen_errors_std_libs_sqlite3_connect_directory_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sqlite3"
# dimension = "errors"
# case = "connect_directory_raises"
# subject = "sqlite3.connect"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sqlite3.connect: connect_directory_raises (errors)."""
import sqlite3

_raised = False
try:
    sqlite3.connect("/")
except sqlite3.OperationalError:
    _raised = True
assert _raised, "connect_directory_raises: expected sqlite3.OperationalError"
print("connect_directory_raises OK")
"###);
    assert_output(&out, r###"connect_directory_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/sqlite3/missing_table_raises.py`.
#[test]
fn test_gen_errors_std_libs_sqlite3_missing_table_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sqlite3"
# dimension = "errors"
# case = "missing_table_raises"
# subject = "sqlite3.Connection"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sqlite3.Connection: missing_table_raises (errors)."""
import sqlite3

_raised = False
try:
    sqlite3.connect(":memory:").execute("SELECT * FROM nope_table")
except sqlite3.OperationalError:
    _raised = True
assert _raised, "missing_table_raises: expected sqlite3.OperationalError"
print("missing_table_raises OK")
"###);
    assert_output(&out, r###"missing_table_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/sqlite3/not_null_constraint_raises_integrityerror.py`.
#[test]
fn test_gen_errors_std_libs_sqlite3_not_null_constraint_raises_integrityerror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sqlite3"
# dimension = "errors"
# case = "not_null_constraint_raises_integrityerror"
# subject = "sqlite3.Cursor"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sqlite3.Cursor: inserting NULL into a NOT NULL column raises sqlite3.IntegrityError"""
import sqlite3

conn = sqlite3.connect(":memory:")
cur = conn.cursor()
cur.execute("CREATE TABLE u (id INTEGER PRIMARY KEY, name TEXT NOT NULL)")
_raised = False
try:
    cur.execute("INSERT INTO u (id, name) VALUES (?, ?)", (1, None))
except sqlite3.IntegrityError:
    _raised = True
assert _raised, "NULL into NOT NULL column raises IntegrityError"
conn.close()

print("not_null_constraint_raises_integrityerror OK")
"###);
    assert_output(&out, r###"not_null_constraint_raises_integrityerror OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/sqlite3/primary_key_violation_raises_integrityerror.py`.
#[test]
fn test_gen_errors_std_libs_sqlite3_primary_key_violation_raises_integrityerror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sqlite3"
# dimension = "errors"
# case = "primary_key_violation_raises_integrityerror"
# subject = "sqlite3.Cursor"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sqlite3.Cursor: inserting a duplicate PRIMARY KEY value raises sqlite3.IntegrityError (a DatabaseError subclass)"""
import sqlite3

conn = sqlite3.connect(":memory:")
cur = conn.cursor()
cur.execute("CREATE TABLE pk_test (id INTEGER PRIMARY KEY, name TEXT)")
cur.execute("INSERT INTO pk_test VALUES (1, 'first')")
_raised = False
try:
    cur.execute("INSERT INTO pk_test VALUES (1, 'duplicate')")
except sqlite3.IntegrityError:
    _raised = True
assert _raised, "duplicate primary key raises IntegrityError"
conn.close()

print("primary_key_violation_raises_integrityerror OK")
"###);
    assert_output(&out, r###"primary_key_violation_raises_integrityerror OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/sqlite3/wrong_param_count_raises_programmingerror.py`.
#[test]
fn test_gen_errors_std_libs_sqlite3_wrong_param_count_raises_programmingerror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sqlite3"
# dimension = "errors"
# case = "wrong_param_count_raises_programmingerror"
# subject = "sqlite3.Cursor"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sqlite3.Cursor: binding the wrong number of '?' parameters raises sqlite3.ProgrammingError"""
import sqlite3

conn = sqlite3.connect(":memory:")
cur = conn.cursor()
cur.execute("CREATE TABLE t (a INTEGER, b TEXT)")
_raised = False
try:
    # Two '?' placeholders but only one bound value.
    cur.execute("INSERT INTO t VALUES (?, ?)", (1,))
except sqlite3.ProgrammingError:
    _raised = True
assert _raised, "wrong parameter count raises ProgrammingError"
conn.close()

print("wrong_param_count_raises_programmingerror OK")
"###);
    assert_output(&out, r###"wrong_param_count_raises_programmingerror OK
"###);
}
