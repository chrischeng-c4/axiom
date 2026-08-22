use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/std-libs/regression/regression_tests__test_auto_commit.py`.
#[test]
fn test_gen_behavior_std_libs_regression_regression_tests__test_auto_commit() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "regression"
# dimension = "behavior"
# case = "regression_tests__test_auto_commit"
# subject = "cpython.test_regression.RegressionTests.test_auto_commit"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_sqlite3/test_regression.py"
# status = "filled"
# ///
import datetime
import sqlite3 as sqlite
import weakref
import functools
self_con = sqlite.connect(':memory:')
'\n        Verifies that creating a connection in autocommit mode works.\n        2.5.3 introduced a regression so that these could no longer\n        be created.\n        '
con = sqlite.connect(':memory:', isolation_level=None)

print("RegressionTests::test_auto_commit: ok")
"###);
    assert_output(&out, r###"RegressionTests::test_auto_commit: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/regression/regression_tests__test_bpo37347.py`.
#[test]
fn test_gen_behavior_std_libs_regression_regression_tests__test_bpo37347() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "regression"
# dimension = "behavior"
# case = "regression_tests__test_bpo37347"
# subject = "cpython.test_regression.RegressionTests.test_bpo37347"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_sqlite3/test_regression.py"
# status = "filled"
# ///
import datetime
import sqlite3 as sqlite
import weakref
import functools
self_con = sqlite.connect(':memory:')

class Printer:

    def log(self, *args):
        return sqlite.SQLITE_OK
for method in [self_con.set_trace_callback, functools.partial(self_con.set_progress_handler, n=1), self_con.set_authorizer]:
    printer_instance = Printer()
    method(printer_instance.log)
    method(printer_instance.log)
    self_con.execute('select 1')
    method(None)

print("RegressionTests::test_bpo37347: ok")
"###);
    assert_output(&out, r###"RegressionTests::test_bpo37347: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/regression/regression_tests__test_empty_statement.py`.
#[test]
fn test_gen_behavior_std_libs_regression_regression_tests__test_empty_statement() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "regression"
# dimension = "behavior"
# case = "regression_tests__test_empty_statement"
# subject = "cpython.test_regression.RegressionTests.test_empty_statement"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_sqlite3/test_regression.py"
# status = "filled"
# ///
import datetime
import sqlite3 as sqlite
import weakref
import functools
self_con = sqlite.connect(':memory:')
'\n        pysqlite used to segfault with SQLite versions 3.5.x. These return NULL\n        for "no-operation" statements\n        '
self_con.execute('')

print("RegressionTests::test_empty_statement: ok")
"###);
    assert_output(&out, r###"RegressionTests::test_empty_statement: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/regression/regression_tests__test_pragma_autocommit.py`.
#[test]
fn test_gen_behavior_std_libs_regression_regression_tests__test_pragma_autocommit() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "regression"
# dimension = "behavior"
# case = "regression_tests__test_pragma_autocommit"
# subject = "cpython.test_regression.RegressionTests.test_pragma_autocommit"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_sqlite3/test_regression.py"
# status = "filled"
# ///
import datetime
import sqlite3 as sqlite
import weakref
import functools
self_con = sqlite.connect(':memory:')
'\n        Verifies that running a PRAGMA statement that does an autocommit does\n        work. This did not work in 2.5.3/2.5.4.\n        '
cur = self_con.cursor()
cur.execute('create table foo(bar)')
cur.execute('insert into foo(bar) values (5)')
cur.execute('pragma page_size')
row = cur.fetchone()

print("RegressionTests::test_pragma_autocommit: ok")
"###);
    assert_output(&out, r###"RegressionTests::test_pragma_autocommit: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/regression/regression_tests__test_pragma_schema_version.py`.
#[test]
fn test_gen_behavior_std_libs_regression_regression_tests__test_pragma_schema_version() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "regression"
# dimension = "behavior"
# case = "regression_tests__test_pragma_schema_version"
# subject = "cpython.test_regression.RegressionTests.test_pragma_schema_version"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_sqlite3/test_regression.py"
# status = "filled"
# ///
import datetime
import sqlite3 as sqlite
import weakref
import functools
self_con = sqlite.connect(':memory:')
con = sqlite.connect(':memory:', detect_types=sqlite.PARSE_COLNAMES)
try:
    cur = self_con.cursor()
    cur.execute('pragma schema_version')
finally:
    cur.close()
    con.close()

print("RegressionTests::test_pragma_schema_version: ok")
"###);
    assert_output(&out, r###"RegressionTests::test_pragma_schema_version: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/regression/regression_tests__test_pragma_user_version.py`.
#[test]
fn test_gen_behavior_std_libs_regression_regression_tests__test_pragma_user_version() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "regression"
# dimension = "behavior"
# case = "regression_tests__test_pragma_user_version"
# subject = "cpython.test_regression.RegressionTests.test_pragma_user_version"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_sqlite3/test_regression.py"
# status = "filled"
# ///
import datetime
import sqlite3 as sqlite
import weakref
import functools
self_con = sqlite.connect(':memory:')
cur = self_con.cursor()
cur.execute('pragma user_version')

print("RegressionTests::test_pragma_user_version: ok")
"###);
    assert_output(&out, r###"RegressionTests::test_pragma_user_version: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/regression/regression_tests__test_statement_finalization_on_close_db.py`.
#[test]
fn test_gen_behavior_std_libs_regression_regression_tests__test_statement_finalization_on_close_db() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "regression"
# dimension = "behavior"
# case = "regression_tests__test_statement_finalization_on_close_db"
# subject = "cpython.test_regression.RegressionTests.test_statement_finalization_on_close_db"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_sqlite3/test_regression.py"
# status = "filled"
# ///
import datetime
import sqlite3 as sqlite
import weakref
import functools
self_con = sqlite.connect(':memory:')
con = sqlite.connect(':memory:')
cursors = []
for i in range(105):
    cur = con.cursor()
    cursors.append(cur)
    cur.execute('select 1 x union select ' + str(i))
con.close()

print("RegressionTests::test_statement_finalization_on_close_db: ok")
"###);
    assert_output(&out, r###"RegressionTests::test_statement_finalization_on_close_db: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/regression/regression_tests__test_statement_reset.py`.
#[test]
fn test_gen_behavior_std_libs_regression_regression_tests__test_statement_reset() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "regression"
# dimension = "behavior"
# case = "regression_tests__test_statement_reset"
# subject = "cpython.test_regression.RegressionTests.test_statement_reset"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_sqlite3/test_regression.py"
# status = "filled"
# ///
import datetime
import sqlite3 as sqlite
import weakref
import functools
self_con = sqlite.connect(':memory:')
con = sqlite.connect(':memory:', cached_statements=5)
cursors = [con.cursor() for x in range(5)]
cursors[0].execute('create table test(x)')
for i in range(10):
    cursors[0].executemany('insert into test(x) values (?)', [(x,) for x in range(10)])
for i in range(5):
    cursors[i].execute(' ' * i + 'select x from test')
con.rollback()

print("RegressionTests::test_statement_reset: ok")
"###);
    assert_output(&out, r###"RegressionTests::test_statement_reset: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/regression/regression_tests__test_str_subclass.py`.
#[test]
fn test_gen_behavior_std_libs_regression_regression_tests__test_str_subclass() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "regression"
# dimension = "behavior"
# case = "regression_tests__test_str_subclass"
# subject = "cpython.test_regression.RegressionTests.test_str_subclass"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_sqlite3/test_regression.py"
# status = "filled"
# ///
import datetime
import sqlite3 as sqlite
import weakref
import functools
self_con = sqlite.connect(':memory:')
"\n        The Python 3.0 port of the module didn't cope with values of subclasses of str.\n        "

class MyStr(str):
    pass
self_con.execute('select ?', (MyStr('abc'),))

print("RegressionTests::test_str_subclass: ok")
"###);
    assert_output(&out, r###"RegressionTests::test_str_subclass: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/regression/regression_tests__test_workaround_for_buggy_sqlite_transfer_bindings.py`.
#[test]
fn test_gen_behavior_std_libs_regression_regression_tests__test_workaround_for_buggy_sqlite_transfer_bindings() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "regression"
# dimension = "behavior"
# case = "regression_tests__test_workaround_for_buggy_sqlite_transfer_bindings"
# subject = "cpython.test_regression.RegressionTests.test_workaround_for_buggy_sqlite_transfer_bindings"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_sqlite3/test_regression.py"
# status = "filled"
# ///
import datetime
import sqlite3 as sqlite
import weakref
import functools
self_con = sqlite.connect(':memory:')
'\n        pysqlite would crash with older SQLite versions unless\n        a workaround is implemented.\n        '
self_con.execute('create table foo(bar)')
self_con.execute('drop table foo')
self_con.execute('create table foo(bar)')

print("RegressionTests::test_workaround_for_buggy_sqlite_transfer_bindings: ok")
"###);
    assert_output(&out, r###"RegressionTests::test_workaround_for_buggy_sqlite_transfer_bindings: ok
"###);
}
