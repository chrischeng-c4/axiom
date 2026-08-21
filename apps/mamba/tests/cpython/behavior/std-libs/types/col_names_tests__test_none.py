# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "behavior"
# case = "col_names_tests__test_none"
# subject = "cpython.test_types.ColNamesTests.test_none"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_sqlite3/test_types.py"
# status = "filled"
# ///
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
self_cur.execute('insert into test(x) values (?)', (None,))
self_cur.execute('select x from test')
val = self_cur.fetchone()[0]
assert val == None

print("ColNamesTests::test_none: ok")
