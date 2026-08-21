# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "backup"
# dimension = "behavior"
# case = "backup_tests__test_progress_all_pages_at_once_1_ucb39068"
# subject = "cpython.test_backup.BackupTests.test_progress_all_pages_at_once_1"
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
    journal.append(remaining)
with sqlite.connect(':memory:') as bck:
    self_cx.backup(bck, progress=progress)
    verify_backup(bck)
assert len(journal) == 1
assert journal[0] == 0

print("BackupTests::test_progress_all_pages_at_once_1: ok")
