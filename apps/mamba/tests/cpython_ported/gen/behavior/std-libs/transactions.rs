use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/std-libs/transactions/special_command_tests__test_drop_table.py`.
#[test]
fn test_gen_behavior_std_libs_transactions_special_command_tests__test_drop_table() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "transactions"
# dimension = "behavior"
# case = "special_command_tests__test_drop_table"
# subject = "cpython.test_transactions.SpecialCommandTests.test_drop_table"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_sqlite3/test_transactions.py"
# status = "filled"
# ///
import sqlite3 as sqlite
from contextlib import contextmanager
self_con = sqlite.connect(':memory:')
self_cur = self_con.cursor()
self_cur.execute('create table test(i)')
self_cur.execute('insert into test(i) values (5)')
self_cur.execute('drop table test')

print("SpecialCommandTests::test_drop_table: ok")
"###);
    assert_output(&out, r###"SpecialCommandTests::test_drop_table: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/transactions/special_command_tests__test_pragma.py`.
#[test]
fn test_gen_behavior_std_libs_transactions_special_command_tests__test_pragma() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "transactions"
# dimension = "behavior"
# case = "special_command_tests__test_pragma"
# subject = "cpython.test_transactions.SpecialCommandTests.test_pragma"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_sqlite3/test_transactions.py"
# status = "filled"
# ///
import sqlite3 as sqlite
from contextlib import contextmanager
self_con = sqlite.connect(':memory:')
self_cur = self_con.cursor()
self_cur.execute('create table test(i)')
self_cur.execute('insert into test(i) values (5)')
self_cur.execute('pragma count_changes=1')

print("SpecialCommandTests::test_pragma: ok")
"###);
    assert_output(&out, r###"SpecialCommandTests::test_pragma: ok
"###);
}
