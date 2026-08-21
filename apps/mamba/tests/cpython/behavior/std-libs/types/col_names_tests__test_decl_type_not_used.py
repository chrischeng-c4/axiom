# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "behavior"
# case = "col_names_tests__test_decl_type_not_used"
# subject = "cpython.test_types.ColNamesTests.test_decl_type_not_used"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_sqlite3/test_types.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import datetime
import sqlite3 as sqlite
import sys
self_con = sqlite.connect(':memory:', detect_types=sqlite.PARSE_COLNAMES)
self_cur = self_con.cursor()
self_cur.execute('create table test(x foo)')
sqlite.converters['FOO'] = lambda x: '[%s]' % x.decode('ascii')
sqlite.converters['BAR'] = lambda x: '<%s>' % x.decode('ascii')
sqlite.converters['EXC'] = lambda x: 5 / 0
sqlite.converters['B1B1'] = lambda x: 'MARKER'
'\n        Assures that the declared type is not used when PARSE_DECLTYPES\n        is not set.\n        '
self_cur.execute('insert into test(x) values (?)', ('xxx',))
self_cur.execute('select x from test')
val = self_cur.fetchone()[0]
assert val == 'xxx'

print("ColNamesTests::test_decl_type_not_used: ok")
