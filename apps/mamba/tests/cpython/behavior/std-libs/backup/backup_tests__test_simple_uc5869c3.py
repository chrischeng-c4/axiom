# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "backup"
# dimension = "behavior"
# case = "backup_tests__test_simple_uc5869c3"
# subject = "cpython.test_backup.BackupTests.test_simple"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_sqlite3/test_backup.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import sqlite3 as sqlite

def verify_backup(bckcx):
    result = bckcx.execute('SELECT key FROM foo ORDER BY key').fetchall()
    assert result[0][0] == 3
    assert result[1][0] == 4
cx = self_cx = sqlite.connect(':memory:')
cx.execute('CREATE TABLE foo (key INTEGER)')
cx.executemany('INSERT INTO foo (key) VALUES (?)', [(3,), (4,)])
cx.commit()
with sqlite.connect(':memory:') as bck:
    self_cx.backup(bck)
    verify_backup(bck)

print("BackupTests::test_simple: ok")
