# /// script
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
