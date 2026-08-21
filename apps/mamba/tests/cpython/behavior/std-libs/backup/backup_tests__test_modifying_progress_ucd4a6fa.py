# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "backup"
# dimension = "behavior"
# case = "backup_tests__test_modifying_progress_ucd4a6fa"
# subject = "cpython.test_backup.BackupTests.test_modifying_progress"
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
journal = []

def progress(status, remaining, total):
    if not journal:
        self_cx.execute('INSERT INTO foo (key) VALUES (?)', (remaining + 1000,))
        self_cx.commit()
    journal.append(remaining)
with sqlite.connect(':memory:') as bck:
    self_cx.backup(bck, pages=1, progress=progress)
    verify_backup(bck)
    result = bck.execute('SELECT key FROM foo WHERE key >= 1000 ORDER BY key').fetchall()
    assert result[0][0] == 1001
assert len(journal) == 3
assert journal[0] == 1
assert journal[1] == 1
assert journal[2] == 0

print("BackupTests::test_modifying_progress: ok")
